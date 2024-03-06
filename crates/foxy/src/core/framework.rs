use std::{sync::Arc, thread::JoinHandle, time::Duration};

use crossbeam::{channel::TryRecvError, queue::ArrayQueue};
use egui::RawInput;
use ezwin::prelude::*;
use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_time::{timer::Timer, Time};
use foxy_utils::mailbox::{Mailbox, MessagingError};
use tracing::*;

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
  window: Arc<Window>,
  preferred_visibility: Visibility,

  renderer: Renderer,
  render_time: Time,
  render_queue: Arc<ArrayQueue<RenderData>>,
  render_mailbox: Mailbox<RenderLoopMessage, GameLoopMessage>,

  game_thread: Option<JoinHandle<FoxyResult<()>>>,
  fps_timer: Timer,

  debug_info: DebugInfo,
  frame_count: u32,
}

impl Framework {
  pub fn new<App: Runnable>(mut settings: FoxySettings) -> FoxyResult<Self> {
    let preferred_visibility = settings.window.visibility;
    settings.window.visibility = Visibility::Hidden;
    settings.window.close_on_x = false;

    let window = Window::new(settings.window.clone())?;

    Self::initialize::<App>(settings, window, preferred_visibility)
  }
}

impl Framework {
  const GAME_THREAD_ID: &'static str = "foxy";
  const MAX_FRAME_DATA_IN_FLIGHT: usize = 2;

  fn initialize<App: Runnable>(
    settings: FoxySettings,
    window: Arc<Window>,
    preferred_visibility: Visibility,
  ) -> FoxyResult<Self> {
    trace!("Firing up Foxy");

    let renderer = Renderer::new(window.clone())?;
    let render_time = settings.time.build();
    let render_queue = Arc::new(ArrayQueue::new(Self::MAX_FRAME_DATA_IN_FLIGHT));

    let time = settings.time.build();
    let foxy = Foxy::new(time, window.clone());
    let (game_mailbox, render_mailbox) = Mailbox::new_entangled_pair();
    let game_thread = Some(Self::game_loop::<App>(game_mailbox, foxy, render_queue.clone())?);

    Ok(Self {
      window,
      preferred_visibility,
      renderer,
      render_time,
      render_queue,
      render_mailbox,
      game_thread,
      debug_info: settings.debug_info,
      fps_timer: Timer::new(),
      frame_count: 0,
    })
  }

  fn exit(&mut self) {
    trace!("Exiting");
    self.window.close();
    if let Some(thread) = self.game_thread.take() {
      let _ = thread.join();
    }
  }

  pub fn run(self) -> FoxyResult<()> {
    info!("KON KON KITSUNE!");

    debug!("Kicking off render loop");

    let window = self.window.clone();
    window.run(self);

    debug!("Wrapping up render loop");

    info!("OTSU KON DESHITA!");

    Ok(())
  }

  fn render(&mut self) {
    let render_data = self.render_queue.pop();
    let Some(render_data) = render_data else {
      return;
    };

    self.render_time.update();
    while self.render_time.should_do_tick_unchecked() {
      self.render_time.tick();
    }

    if let Err(error) = self.renderer.render(&self.render_time, render_data) {
      error!("`{error}` Aborting...");
      let _ = self.render_mailbox.send_and_recv(RenderLoopMessage::MustExit);
      self.exit();
    }

    if self.fps_timer.has_elapsed(Duration::from_millis(200)) {
      if let DebugInfo::Shown = self.debug_info {
        let ft = self.render_time.average_delta_secs();
        self
          .window
          .set_subtitle(format!(" | {:^5.4} s | {:>5.0} FPS", ft, 1.0 / ft));
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
        debug!("Kicking off game loop");

        let mut app = App::new(&mut foxy);
        app.start(&mut foxy);
        loop {
          let next_message = mailbox.try_recv();

          let raw_input: RawInput = foxy.take_egui_raw_input();

          match next_message {
            Ok(message) => match message {
              RenderLoopMessage::Window(window_message) => {
                foxy.time.update();
                while foxy.time.should_do_tick_unchecked() {
                  foxy.time.tick();
                  app.fixed_update(&mut foxy, &window_message);
                }

                app.update(&mut foxy, &window_message);

                let _full_output = foxy.egui_context.run(raw_input, |ui| {
                  app.egui(&foxy, ui);
                });

                render_queue.force_push(RenderData {});
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
                }
              }
              _ => (),
            },
            Err(MessagingError::TryRecvError {
              error: TryRecvError::Disconnected,
            }) => {
              app.stop(&mut foxy);
              break;
            }
            _ => (),
          };
        }

        trace!("Exiting game!");

        app.delete();

        debug!("Wrapping up game loop");
        Ok(())
      })?;

    Ok(handle)
  }
}

impl WindowCallback for Framework {
  fn on_message(&mut self, _window: &Arc<Window>, message: Message) {
    let was_handled = self.renderer.input(&message);
    if was_handled {
      return;
    }

    match &message {
      Message::Window(WindowMessage::Resized { .. }) => {
        self.renderer.resize();
        self.render();
      }
      Message::Window(WindowMessage::CloseRequested) => {
        trace!("Close requested");

        if let Err(MessagingError::SendError { .. }) = self.render_mailbox.send(RenderLoopMessage::ExitRequested) {
          panic!("game loop disconnected before exit message was sent");
        }

        let response = self.render_mailbox.recv().unwrap();

        trace!("Evaluating close response");

        if let GameLoopMessage::Exit = response {
          self.exit();
        }
      }
      Message::Window(WindowMessage::Closing) => {
        trace!("Closing window!");
        self.renderer.delete();
      }
      Message::Window(WindowMessage::Draw) => {
        self.render();
      }
      _ => (),
    }

    if !self.window.is_closing() {
      if let Err(error) = self.render_mailbox.try_send(RenderLoopMessage::Window(message)) {
        error!("{error}")
      }
      self.window.request_redraw();
    }

    self.frame_count = self.frame_count.wrapping_add(1);
    if self.frame_count > 10 {
      self.window.set_visibility(self.preferred_visibility);
    }
  }
}
