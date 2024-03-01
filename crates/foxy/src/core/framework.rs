use std::{
  sync::{Arc, Barrier},
  thread::JoinHandle,
  time::Duration,
};

use crossbeam::{channel::TryRecvError, queue::ArrayQueue};
use egui::RawInput;
use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_time::{timer::Timer, EngineTime};
use foxy_utils::mailbox::{Mailbox, MessagingError};
use tracing::*;
use winit::{
  event::KeyEvent,
  event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget},
  keyboard::PhysicalKey,
  window::{Window, WindowBuilder},
};

use super::{
  builder::{DebugInfo, FoxySettings},
  runnable::Runnable,
  state::Foxy,
  FoxyResult,
};
use crate::core::{
  message::{GameLoopMessage, RenderLoopMessage},
  runnable::Flow,
};

pub struct Framework {
  event_loop: Option<EventLoop<()>>,
  window: Arc<Window>,
  title: String,
  preferred_visibility: bool,
  should_wait: bool,

  renderer: Renderer,
  render_time: EngineTime,
  render_queue: Arc<ArrayQueue<RenderData>>,
  render_mailbox: Mailbox<RenderLoopMessage, GameLoopMessage>,

  game_thread: Option<JoinHandle<FoxyResult<()>>>,
  fps_timer: Timer,

  debug_info: DebugInfo,
  frame_count: u32,

  sync_barrier: Arc<Barrier>,
}

impl Framework {
  pub fn new<App: Runnable>(settings: FoxySettings) -> FoxyResult<Self> {
    Self::with_events::<App>(settings)
  }
}

impl Framework {
  const GAME_THREAD_ID: &'static str = "foxy";

  // A relic of ancient, more flexible times
  pub fn with_events<App: Runnable>(mut settings: FoxySettings) -> FoxyResult<Self> {
    trace!("Firing up Foxy");
    let preferred_visibility = settings.window.is_visible;
    settings.window.is_visible = false;
    settings.window.should_close_on_x = false;

    let event_loop = EventLoop::new()?;

    let window = Arc::new(
      WindowBuilder::new()
        .with_title(settings.window.title.clone())
        .with_inner_size(settings.window.size)
        .with_visible(settings.window.is_visible)
        .build(&event_loop)?,
    );

    let renderer = Renderer::new(window.clone())?;
    let render_time = settings.time.build();
    let render_queue = Arc::new(ArrayQueue::new(Renderer::MAX_FRAMES_IN_FLIGHT));

    let sync_barrier = Arc::new(Barrier::new(2));

    let time = settings.time.build();
    let foxy = Foxy::new(time, window.clone());
    let (game_mailbox, render_mailbox) = Mailbox::new_entangled_pair();
    let game_thread = Some(Self::game_loop::<App>(
      game_mailbox,
      foxy,
      render_queue.clone(),
      sync_barrier.clone(),
    )?);

    Ok(Self {
      event_loop: Some(event_loop),
      window,
      title: settings.window.title,
      preferred_visibility,
      should_wait: settings.window.should_wait,
      renderer,
      render_time,
      render_queue,
      render_mailbox,
      game_thread,
      debug_info: settings.debug_info,
      fps_timer: Timer::new(),
      frame_count: 0,
      sync_barrier,
    })
  }

  fn exit(&mut self, elwt: &EventLoopWindowTarget<()>) {
    trace!("Exiting");
    elwt.exit();
    if let Some(thread) = self.game_thread.take() {
      let _ = thread.join();
    }
    self.renderer.delete();
  }

  pub fn run(mut self) -> FoxyResult<()> {
    info!("KON KON KITSUNE!");

    debug!("Kicking off render loop");
    let event_loop = self.event_loop.take().unwrap();

    event_loop.set_control_flow(match self.should_wait {
      false => ControlFlow::Poll,
      true => ControlFlow::Wait,
    });

    event_loop.run(|event, elwt| match event {
      winit::event::Event::WindowEvent {
        event: window_event, ..
      } => {
        let was_handled = self.renderer.input(&window_event);
        if was_handled {
          return;
        }

        match &window_event {
          winit::event::WindowEvent::Resized(..) | winit::event::WindowEvent::ScaleFactorChanged { .. } => {
            self.renderer.resize();
            self.render(elwt);
          }
          winit::event::WindowEvent::CloseRequested => {
            trace!("Close requested");

            if let Err(MessagingError::SendError { .. }) = self.render_mailbox.send(RenderLoopMessage::ExitRequested) {
              error!("game loop disconnected before exit message was sent");
              self.exit(elwt);
            }

            let response = loop {
              match self.render_mailbox.try_recv() {
                Ok(response) => {
                  break response;
                }
                Err(MessagingError::TryRecvError {
                  error: TryRecvError::Disconnected,
                }) => {
                  error!("game loop disconnected before exit response was recieved");
                  self.exit(elwt);
                }
                _ => (),
              };
            };

            trace!("Evaluating exit response");

            if let GameLoopMessage::Exit = response {
              self.exit(elwt);
            }
          }
          winit::event::WindowEvent::RedrawRequested => {
            self.render(elwt);
          }
          _ => (),
        }

        if !elwt.exiting() {
          if let Err(error) = self.render_mailbox.try_send(RenderLoopMessage::Window(window_event)) {
            error!("{error}")
          }
        }
      }
      winit::event::Event::AboutToWait => match self.frame_count {
        0..=9 => self.render(elwt),
        10 => self.window.set_visible(self.preferred_visibility),
        11.. => self.window.request_redraw(),
      },
      _ => (),
    })?;

    debug!("Wrapping up render loop");

    info!("OTSU KON DESHITA!");

    Ok(())
  }

  fn render(&mut self, elwt: &EventLoopWindowTarget<()>) {
    let render_data = self.render_queue.pop();
    let Some(render_data) = render_data else {
      return;
    };

    self.render_time.update();
    while self.render_time.should_do_tick_unchecked() {
      self.render_time.tick();
    }

    if let Err(error) = self.renderer.render(self.render_time.time(), render_data) {
      error!("`{error}` Aborting...");
      let _ = self.render_mailbox.send_and_recv(RenderLoopMessage::MustExit);
      self.exit(elwt);
    }

    if self.fps_timer.has_elapsed(Duration::from_millis(200)) {
      if let DebugInfo::Shown = self.debug_info {
        let time = self.render_time.time();
        let ft = time.average_delta_secs();
        self
          .window
          .set_title(format!("{} | {:^5.4} s | {:>5.0} FPS", self.title, ft, 1.0 / ft).as_str());
      }
    }

    self.frame_count = self.frame_count.wrapping_add(1);
  }

  fn game_loop<App: Runnable>(
    mailbox: Mailbox<GameLoopMessage, RenderLoopMessage>,
    mut foxy: Foxy,
    render_queue: Arc<ArrayQueue<RenderData>>,
    sync_barrier: Arc<Barrier>,
  ) -> FoxyResult<JoinHandle<FoxyResult<()>>> {
    let handle = std::thread::Builder::new()
      .name(Self::GAME_THREAD_ID.into())
      .spawn(move || -> FoxyResult<()> {
        debug!("Kicking off game loop");

        let mut app = App::new(&mut foxy);
        app.start(&mut foxy);
        loop {
          let next_message = mailbox.try_recv();

          let raw_input: RawInput = foxy.take_egui_raw_input();

          let message = match next_message {
            Ok(message) => match message {
              RenderLoopMessage::Window(window_message) => {
                match window_message {
                  winit::event::WindowEvent::KeyboardInput {
                    event:
                      KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        repeat,
                        ..
                      },
                    ..
                  } => {
                    foxy.input().update_key_state(key_code, state, repeat);
                  }
                  winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                    foxy.input().update_modifiers_state(modifiers);
                  }
                  winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    foxy.input().update_mouse_button_state(button, state);
                  }
                  _ => (),
                }

                Some(window_message)
              }
              RenderLoopMessage::MustExit => {
                app.stop(&mut foxy);
                let _ = mailbox.send(GameLoopMessage::Exit);
                break;
              }
              RenderLoopMessage::ExitRequested => {
                if let Flow::Exit = app.stop(&mut foxy) {
                  let _ = mailbox.send(GameLoopMessage::Exit);
                  break;
                } else {
                  let _ = mailbox.send(GameLoopMessage::DontExit);
                  None
                }
              }
              _ => None,
            },
            Err(MessagingError::TryRecvError {
              error: TryRecvError::Disconnected,
            }) => {
              app.stop(&mut foxy);
              break;
            }
            _ => None,
          };

          foxy.time.update();
          while foxy.time.should_do_tick_unchecked() {
            foxy.time.tick();
            app.fixed_update(&mut foxy, message.as_ref());
          }

          app.update(&mut foxy, message.as_ref());

          app.late_update(&mut foxy, message.as_ref());

          let _full_output = foxy.egui_context.run(raw_input, |ui| {
            app.egui(&foxy, ui);
          });

          render_queue.force_push(RenderData {});
        }

        trace!("Exiting game!");

        app.delete();

        debug!("Wrapping up game loop");
        Ok(())
      })?;

    Ok(handle)
  }
}
