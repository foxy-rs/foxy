use std::{sync::Arc, thread::JoinHandle, time::Duration};

use crossbeam::{channel::TryRecvError, queue::ArrayQueue};
use ezwin::{
  prelude::{Message, Window, WindowMessage},
  window::settings::Visibility,
};
use foxy_log::LogErr;
use foxy_renderer::{
  error::RendererError,
  renderer::{render_data::RenderData, Renderer},
};
use foxy_time::{timer::Timer, EngineTime};
use foxy_utils::mailbox::{Mailbox, MessagingError};
use tracing::{debug, error, info, trace, warn};

use super::{
  builder::{DebugInfo, FoxySettings},
  engine_state::Foxy,
  runnable::Runnable,
  FoxyResult,
};
use crate::core::{
  message::{GameLoopMessage, RenderLoopMessage},
  runnable::Flow,
};

pub struct Framework {
  renderer: Renderer,
  render_time: EngineTime,
  render_queue: Arc<ArrayQueue<RenderData>>,
  render_mailbox: Mailbox<RenderLoopMessage, GameLoopMessage>,

  // Keep window below the renderer to ensure proper drop order
  window: Arc<Window>,
  preferred_visibility: Visibility,

  game_thread: Option<JoinHandle<FoxyResult<()>>>,
  fps_timer: Timer,

  debug_info: DebugInfo,
  had_first_frame: bool,
}

impl Framework {
  pub fn new<App: Runnable>(settings: FoxySettings) -> FoxyResult<Self> {
    Self::with_events::<App>(settings)
  }
}

impl Framework {
  const GAME_THREAD_ID: &'static str = "foxy";
  const MAX_FRAME_DATA_IN_FLIGHT: usize = 2;

  // A relic of ancient, more flexible times
  pub fn with_events<App: Runnable>(mut settings: FoxySettings) -> FoxyResult<Self> {
    trace!("Firing up Foxy");
    let preferred_visibility = settings.window.visibility;
    settings.window.visibility = Visibility::Hidden;

    let window = Arc::new(Window::new(settings.window)?);

    let renderer = Renderer::new(window.clone())?;
    let render_time = settings.time.build();
    let render_queue = Arc::new(ArrayQueue::new(Self::MAX_FRAME_DATA_IN_FLIGHT));

    let time = settings.time.build();
    let foxy = Foxy::new(time, window.clone());
    let (game_mailbox, render_mailbox) = Mailbox::new_entangled_pair();
    let game_thread = Some(Self::game_loop::<App>(game_mailbox, foxy, render_queue.clone())?);

    Ok(Self {
      renderer,
      render_time,
      render_queue,
      render_mailbox,
      window,
      preferred_visibility,
      game_thread,
      debug_info: settings.debug_info,
      fps_timer: Timer::new(),
      had_first_frame: false,
    })
  }

  pub fn run(mut self) -> FoxyResult<()> {
    info!("KON KON KITSUNE!");

    let _ = self.render_mailbox.send(RenderLoopMessage::Start).log_error();
    let window = self.window.clone();

    for message in window.as_ref() {
      let was_handled = self.renderer.input(&message);

      if was_handled {
        continue;
      }

      match &message {
        Message::Window(WindowMessage::Resizing { .. }) => {
          self.renderer.resize();
          self.render();
        }
        Message::CloseRequested => {
          let response = self
            .render_mailbox
            .send_and_recv(RenderLoopMessage::ExitRequested)
            .log_error();
          if let Ok(GameLoopMessage::Exit) = response {
            if let Some(thread) = self.game_thread.take() {
              let _ = thread.join();
            }
          }
        }
        Message::Closing => {
          warn!("Closing!");
          self.renderer.delete();
          info!("OTSU KON DESHITA!");
        }
        Message::None => {
          self.render();

          if !self.had_first_frame {
            self.had_first_frame = true;
            window.set_visibility(self.preferred_visibility);
          }
        }
        _ => (),
      }

      if !window.is_closing() {
        if let Err(error) = self.render_mailbox.send(RenderLoopMessage::Window(message)) {
          error!("{error}")
        }
      }
    }

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

    match self.renderer.render(self.render_time.time(), render_data) {
      Ok(()) if !self.had_first_frame => {
        self.had_first_frame = true;
        self.window.set_visibility(Visibility::Shown);
      }
      Err(RendererError::RebuildSwapchain) => {}
      Err(error) => {
        error!("`{error}` Aborting...");
        let _ = self.render_mailbox.send_and_recv(RenderLoopMessage::MustExit);
        self.window.close();
      }
      _ => (),
    }

    if self.fps_timer.has_elapsed(Duration::from_millis(200)) {
      if let DebugInfo::Shown = self.debug_info {
        let time = self.render_time.time();
        let ft = time.average_delta_secs();
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
        let _ = mailbox.recv().log_error(); // wait for startup message
        debug!("Kicking off game loop");

        let mut app = App::new(&mut foxy);
        app.start(&mut foxy);
        loop {
          let next_message = mailbox.try_recv();

          match next_message {
            Ok(message) => match message {
              RenderLoopMessage::Window(window_message) => {
                foxy.time.update();
                while foxy.time.should_do_tick_unchecked() {
                  foxy.time.tick();
                  app.fixed_update(&mut foxy, &window_message);
                }

                app.update(&mut foxy, &window_message);
                render_queue.force_push(RenderData {});
              }
              RenderLoopMessage::MustExit => {
                app.stop(&mut foxy);
                let _ = mailbox.send(GameLoopMessage::Exit);
                app.delete();
                break;
              }
              RenderLoopMessage::ExitRequested => {
                if let Flow::Exit = app.stop(&mut foxy) {
                  let _ = mailbox.send(GameLoopMessage::Exit);
                  app.delete();
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
              app.delete();
              break;
            }
            _ => (),
          };
        }

        debug!("Wrapping up game loop");
        Ok(())
      })?;

    Ok(handle)
  }
}
