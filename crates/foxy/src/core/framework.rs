use std::{sync::Arc, thread::JoinHandle, time::Duration};

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
  engine_state::Foxy,
  runnable::Runnable,
  FoxyResult,
};
use crate::core::{
  event::FoxyEvent,
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

  // Keep window below the renderer to ensure proper drop order
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

    let renderer = Renderer::new(window.clone())?;
    let render_time = create_info.time.build();
    let render_queue = Arc::new(ArrayQueue::new(Self::MAX_FRAME_DATA_IN_FLIGHT));

    let time = create_info.time.build();
    let foxy = Foxy::new(time, window.clone());
    let (game_mailbox, render_mailbox) = Mailbox::new_entangled_pair();
    let game_thread = Some(Self::game_loop::<App>(game_mailbox, foxy, render_queue.clone())?);

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
      return Err(FoxyError::Error(format!("failed to take foxy state")));
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
          let was_handled = state.renderer.input(&event);

          // first check
          if !was_handled {
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
                state.window.request_redraw();
              }
              WindowEvent::RedrawRequested => {
                Self::render(&mut state, elwt);
              }
              _ => (),
            }

            if !elwt.exiting() {
              if let Err(error) = state.render_mailbox.send(RenderLoopMessage::Winit(event)) {
                error!("{error:?}")
              }
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

    match state.renderer.draw(state.render_time.time(), render_data) {
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
    mut foxy: Foxy,
    render_queue: Arc<ArrayQueue<RenderData>>,
  ) -> FoxyResult<JoinHandle<FoxyResult<()>>> {
    let handle = std::thread::Builder::new()
      .name(Self::GAME_THREAD_ID.into())
      .spawn(move || -> FoxyResult<()> {
        let _ = mailbox.recv().log_error(); // wait for startup message
        // debug!("HELLO HELLO BAU BAU");

        let mut app = App::new(&mut foxy);
        app.start(&mut foxy);
        loop {
          let next_message = mailbox.try_recv();

          let event = match next_message {
            Ok(RenderLoopMessage::MustExit) => {
              let _ = mailbox.send(GameLoopMessage::Exit);
              app.stop(&mut foxy);
              app.delete();
              break;
            }
            Ok(RenderLoopMessage::ExitRequested) => {
              if let Flow::Exit = app.stop(&mut foxy) {
                let _ = mailbox.send(GameLoopMessage::Exit);
                app.delete();
                break;
              } else {
                let _ = mailbox.send(GameLoopMessage::DontExit);
              }
              None
            }
            Ok(RenderLoopMessage::Winit(event)) => {
              match event {
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
                  foxy.input.update_key_state(physical_key, element_state, repeat);
                }
                WindowEvent::MouseInput {
                  button,
                  state: element_state,
                  ..
                } => {
                  foxy.input.update_mouse_button_state(button, element_state);
                }
                WindowEvent::ModifiersChanged(mods) => {
                  foxy.input.update_modifiers_state(mods);
                }
                _ => (),
              }

              Some(event)
            }
            Err(MessagingError::TryRecvError {
              error: TryRecvError::Disconnected,
            }) => {
              app.stop(&mut foxy);
              app.delete();
              break;
            }
            _ => None,
          };

          // Loop

          let event = FoxyEvent::from(event);

          foxy.time.update();
          while foxy.time.should_do_tick_unchecked() {
            foxy.time.tick();
            app.fixed_update(&mut foxy, &event);
          }

          if let FoxyEvent::Input(event) = &event {
            app.input(&mut foxy, event);
          }

          app.update(&mut foxy, &event);
          render_queue.force_push(RenderData {});

          if let FoxyEvent::Window(event) = &event {
            app.window(&mut foxy, event);
          }
        }

        // debug!("BAU BAU FOR NOW");
        Ok(())
      })?;

    Ok(handle)
  }
}
