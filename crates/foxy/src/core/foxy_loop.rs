use std::{
  sync::{Arc, Mutex},
  thread::JoinHandle,
  time::Duration,
};

use crossbeam::{channel::TryRecvError, queue::ArrayQueue};
use foxy_renderer::{
  error::RendererError,
  renderer::{render_data::RenderData, Renderer},
};
use foxy_utils::{
  log::LogErr,
  mailbox::{Mailbox, MessagingError},
  time::{timer::Timer, EngineTime},
};
use tracing::*;
use winit::{
  event::{Event, KeyEvent, WindowEvent},
  event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
  window::Window,
};

use super::{
  builder::{DebugInfo, FoxyCreateInfo, Polling},
  runnable::Runnable,
  FoxyResult,
};
use crate::core::{
  event::FoxyEvent,
  foxy_state::{self, Foxy},
  message::{GameLoopMessage, RenderLoopMessage},
  runnable::Flow,
  FoxyError,
};

struct State {
  polling_strategy: Polling,
  debug_info: DebugInfo,

  renderer: Renderer,
  render_time: EngineTime,
  render_queue: Arc<ArrayQueue<RenderData>>,
  render_mailbox: Mailbox<RenderLoopMessage, GameLoopMessage>,

  window: Arc<Window>,

  game_thread: Option<JoinHandle<FoxyResult<()>>>,

  original_title: String,
  fps_timer: Timer,
  had_first_frame: bool,
}

pub struct Framework<T: 'static + Send + Sync> {
  state: Option<State>,
  event_loop: EventLoop<T>,
}

impl Framework<()> {
  pub fn new<App: Runnable>(create_info: FoxyCreateInfo) -> FoxyResult<Self> {
    Self::with_events::<App>(create_info)
  }
}

impl<T: 'static + Send + Sync> Framework<T> {
  const GAME_THREAD_ID: &'static str = "foxy";
  const MAX_FRAME_DATA_IN_FLIGHT: usize = 2;

  pub fn with_events<App: Runnable>(create_info: FoxyCreateInfo) -> FoxyResult<Self> {
    trace!("Firing up Foxy");

    let (event_loop, window) = create_info.window.create_window()?;
    let window = Arc::new(window);

    let time = create_info.time.build();
    let render_queue = Arc::new(ArrayQueue::new(Self::MAX_FRAME_DATA_IN_FLIGHT));

    let foxy = Foxy::new(foxy_state::State::new(time, window.clone()));
    let egui_context = foxy.as_ref().egui_context.clone();
    let (game_mailbox, render_mailbox) = Mailbox::new_entangled_pair();
    let game_thread = Some(Self::game_loop::<App>(game_mailbox, foxy, render_queue.clone())?);

    let renderer = Renderer::new(window.clone(), egui_context)?;
    let render_time = create_info.time.build();

    Ok(Self {
      state: Some(State {
        polling_strategy: create_info.polling_strategy,
        debug_info: create_info.debug_info,
        renderer,
        render_time,
        render_queue,
        render_mailbox,
        original_title: window.title(),
        window,
        game_thread,
        fps_timer: Timer::new(),
        had_first_frame: false,
      }),
      event_loop,
    })
  }

  pub fn run(self) -> FoxyResult<()> {
    info!("KON KON KITSUNE!");
    let Some(mut state) = self.state else {
      return Err(FoxyError::Error("failed to take foxy state".to_owned()));
    };

    let _ = state.render_mailbox.send(RenderLoopMessage::Start).log_error();

    self.event_loop.set_control_flow(match state.polling_strategy {
      Polling::Poll => ControlFlow::Poll,
      Polling::Wait => ControlFlow::Wait,
    });

    Ok(self.event_loop.run(move |event, elwt| {
      let _ = &state; // ensure state is moved

      match event {
        Event::WindowEvent { event, .. } => {
          match event {
            WindowEvent::CloseRequested => {
              let response = state
                .render_mailbox
                .send_and_recv(RenderLoopMessage::ExitRequested)
                .log_error();
              if let Err(_) | Ok(GameLoopMessage::Exit) = response {
                elwt.exit();
              }
            }
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
              state.renderer.refresh();
            }
            WindowEvent::RedrawRequested => {}
            _ => (),
          }

          if !elwt.exiting() {
            Self::render(&mut state, elwt);
            if let Err(error) = state.render_mailbox.send(RenderLoopMessage::Winit(event)) {
              error!("{error:?}")
            }
          }
        }
        Event::AboutToWait => {
          // redraw
          if !state.had_first_frame {
            Self::render(&mut state, elwt);
          } else {
            state.window.request_redraw();
          }
        }
        Event::LoopExiting => {
          if let Some(thread) = state.game_thread.take() {
            let _ = thread.join();
          }
          info!("OTSU KON DESHITA!");
        }
        _ => (),
      }

      // debug!("Æ")
    })?)
  }

  fn render(state: &mut State, elwt: &EventLoopWindowTarget<T>) {
    let render_data = state.render_queue.pop();
    let Some(render_data) = render_data else {
      return;
    };

    state.render_time.update();
    while state.render_time.should_do_tick_unchecked() {
      state.render_time.tick();
    }

    match state.renderer.render_frame(state.render_time.time(), render_data) {
      Ok(()) if !state.had_first_frame => {
        state.had_first_frame = true;
        state.window.set_visible(true);
      }
      Err(RendererError::RebuildSwapchain) => {
        state.renderer.refresh();
      }
      Err(error) => {
        error!("`{error}` Aborting...");
        let _ = state.render_mailbox.send_and_recv(RenderLoopMessage::MustExit);
        elwt.exit();
      }
      _ => (),
    }

    if state.fps_timer.has_elapsed(Duration::from_millis(200)) {
      if let DebugInfo::Shown = state.debug_info {
        let time = state.render_time.time();
        let ft = time.average_delta_secs();
        state
          .window
          .set_title(&format!("{} | {:^5.4} s | {:>5.0} FPS", state.original_title, ft, 1.0 / ft));
      }
    }
  }

  fn game_loop<App: Runnable>(
    mailbox: Mailbox<GameLoopMessage, RenderLoopMessage>,
    foxy: Foxy,
    render_queue: Arc<ArrayQueue<RenderData>>,
  ) -> FoxyResult<JoinHandle<FoxyResult<()>>> {
    let handle = std::thread::Builder::new()
      .name(Self::GAME_THREAD_ID.into())
      .spawn(move || -> FoxyResult<()> {
        let _ = mailbox.recv().log_error();
        let window = foxy.as_ref().window.clone();

        let mut app = App::new(&foxy);
        app.start(&foxy);
        loop {
          let next_message = mailbox.try_recv();

          let raw_input = foxy.as_mut().take_egui_input();

          let event = match next_message {
            Ok(RenderLoopMessage::MustExit) => {
              let _ = mailbox.send(GameLoopMessage::Exit);
              app.stop(&foxy);
              app.delete();
              break;
            }
            Ok(RenderLoopMessage::ExitRequested) => {
              if let Flow::Exit = app.stop(&foxy) {
                let _ = mailbox.send(GameLoopMessage::Exit);
                app.delete();
                break;
              } else {
                let _ = mailbox.send(GameLoopMessage::DontExit);
              }
              None
            }
            Ok(RenderLoopMessage::Winit(event)) => {
              let was_handled = foxy.as_mut().handle_input(&event);

              if !was_handled {
                match event {
                  WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    foxy.as_mut().egui_context.set_zoom_factor(scale_factor as f32);
                  }
                  WindowEvent::KeyboardInput {
                    event:
                      KeyEvent {
                        physical_key,
                        state: element_state,
                        repeat,
                        ..
                      },
                    ..
                  } => {
                    foxy
                      .as_mut()
                      .input
                      .update_key_state(physical_key, element_state, repeat);
                  }
                  WindowEvent::MouseInput {
                    button,
                    state: element_state,
                    ..
                  } => {
                    foxy.as_mut().input.update_mouse_button_state(button, element_state);
                  }
                  WindowEvent::ModifiersChanged(mods) => {
                    foxy.as_mut().input.update_modifiers_state(mods);
                  }
                  _ => (),
                }

                Some(event)
              } else {
                None
              }
            }
            Err(MessagingError::TryRecvError {
              error: TryRecvError::Disconnected,
            }) => {
              app.stop(&foxy);
              app.delete();
              break;
            }
            _ => None,
          };

          // Loop

          let event = FoxyEvent::from(event);

          // let raw_input = foxy.write().egui_state.take_egui_input(&window);

          foxy.as_mut().engine_time.update();
          while foxy.as_mut().engine_time.should_do_tick_unchecked() {
            foxy.as_mut().engine_time.tick();
            app.fixed_update(&foxy, &event);
          }

          if let FoxyEvent::Input(event) = &event {
            app.input(&foxy, event);
          }

          app.update(&foxy, &event);

          if let FoxyEvent::Window(event) = &event {
            app.window(&foxy, event);
          }

          let full_output = foxy.as_ref().egui_context.run(raw_input, |ui| {
            app.gui(&foxy, ui);
          });

          foxy
            .as_mut()
            .egui_state
            .handle_platform_output(&window, full_output.platform_output.clone());

          let mesh_count = foxy.as_ref().meshes.len();
          let meshes = foxy.as_mut().meshes.drain(0..mesh_count).collect();

          render_queue.force_push(RenderData { full_output, meshes });
        }

        // debug!("BAU BAU FOR NOW");
        Ok(())
      })?;

    Ok(handle)
  }
}
