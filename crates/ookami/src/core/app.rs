use std::thread::JoinHandle;

use self::{
  builder::{AppBuilder, AppCreateInfo, HasSize, HasTitle, MissingSize, MissingTitle},
  lifecycle::Lifecycle,
  time::Time,
};
use super::message::{GameLoopMessage, RenderLoopMessage};
use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_window::prelude::*;
use messaging::Mailbox;
use tracing::*;

pub mod builder;
pub mod lifecycle;
pub mod time;

pub struct App {
  time: Time,
  current_state: Lifecycle,
  window: Window,
  render_thread: Option<JoinHandle<anyhow::Result<()>>>,
  game_mailbox: Mailbox<GameLoopMessage, RenderLoopMessage>,
}

impl App {
  pub const RENDER_THREAD_ID: &'static str = "render";

  pub fn builder() -> AppBuilder<MissingTitle, MissingSize> {
    Default::default()
  }

  pub fn new(app_create_info: AppCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
    let time = Time::new(128.0, 1024);
    let current_state = Lifecycle::Entering;

    let mut window = Window::builder()
      .with_title(app_create_info.title.0)
      .with_size(app_create_info.size.width, app_create_info.size.height)
      .with_color_mode(app_create_info.color_mode)
      .with_close_behavior(app_create_info.close_behavior)
      .with_visibility(Visibility::Hidden)
      .build()?;

    let renderer = Renderer::new(&window)?;
    window.set_visibility(Visibility::Shown);

    let (renderer_mailbox, game_mailbox) = Mailbox::new_entangled_pair();
    let render_thread = Some(Self::render_loop(renderer, renderer_mailbox)?);

    Ok(Self {
      time,
      current_state,
      window,
      render_thread,
      game_mailbox,
    })
  }

  // fn run_internal(mut self) -> anyhow::Result<()> {
  //   let (renderer_mailbox, game_mailbox) = Mailbox::new_entangled_pair();

  //   // to allow double mutable borrow
  //   if let Some(renderer) = self.renderer.take() {
  //     self.render_thread = Some(Self::render_loop(renderer, renderer_mailbox)?);
  //     self.game_loop(game_mailbox)?;
  //   };

  //   Ok(())
  // }

  // fn game_loop(&mut self, mut messenger: Mailbox<GameLoopMessage, RenderLoopMessage>) -> anyhow::Result<()> {
  //   self.lifecycle.start(&self.time, window);

  //   while let Some(message) = window.next() {
  //     // TODO: Rewrite so that Renderer owns Window again. App will just own a WindowState lightweight wrapper
  //     //       that will be updated on sync. Also, it'll be able to make requests to change Window through the
  //     //       Command enum that will be created later.
  //     //  MAKE LIFECYCLE AS EVENTS SENT FROM CANVAS
  //     messenger.send_and_wait(GameLoopMessage::SyncWithRenderer)?;

  //     // Main lifecycle
  //     self.time.update();
  //     self.lifecycle.early_update(&self.time, window, &message);
  //     while self.time.should_do_tick() {
  //       self.time.tick();
  //       self.lifecycle.fixed_update(&self.time, window);
  //     }
  //     self.lifecycle.update(&self.time, window, &message);

  //     if self.game_sync_or_exit(window, &mut messenger, message)? {
  //       break;
  //     }
  //   }

  //   self.lifecycle.stop(&self.time, window);

  //   Ok(())
  // }

  fn game_sync_or_exit(&mut self, received_message: &Option<WindowMessage>) -> anyhow::Result<bool> {
    match received_message {
      Some(WindowMessage::Closed) => {
        self.game_mailbox.send_and_wait(GameLoopMessage::Exit)?;
        if let Err(error) = self
          .render_thread
          .take()
          .expect("render_thread handle should not be None")
          .join()
        {
          error!("{error:?}");
        }
        Ok(true)
      }
      _ => {
        self
          .game_mailbox
          .send_and_wait(GameLoopMessage::RenderData(RenderData {}))?;
        Ok(false)
      }
    }
  }

  fn render_loop(
    mut renderer: Renderer,
    mut messenger: Mailbox<RenderLoopMessage, GameLoopMessage>,
  ) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    std::thread::Builder::new()
      .name(Self::RENDER_THREAD_ID.into())
      .spawn(move || -> anyhow::Result<()> {
        trace!("Beginning render");

        loop {
          if Self::renderer_sync_or_exit(&mut renderer, &mut messenger)? {
            break;
          }

          renderer.render()?;

          if Self::renderer_sync_or_exit(&mut renderer, &mut messenger)? {
            break;
          }
        }

        trace!("Ending render");

        Ok(())
      })
      .map_err(anyhow::Error::from)
  }

  fn renderer_sync_or_exit(
    renderer: &mut Renderer,
    messenger: &mut Mailbox<RenderLoopMessage, GameLoopMessage>,
  ) -> anyhow::Result<bool> {
    match messenger.send_and_wait(RenderLoopMessage::SyncWithGame)? {
      GameLoopMessage::SyncWithRenderer => Ok(false),
      GameLoopMessage::RenderData(data) => {
        renderer.update_render_data(data)?;
        Ok(false)
      }
      GameLoopMessage::Exit => Ok(true),
    }
  }

  pub fn time(&self) -> &Time {
    &self.time
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  // pub fn run(self) {
  //   trace!("uchi uchi, uchi da yo");
  //   if let Err(error) = self.run_internal() {
  //     error!("{error}");
  //   }
  //   trace!("otsu mion");
  // }

  pub fn poll(&mut self) -> Option<&Lifecycle> {
    let new_state = match &mut self.current_state {
      Lifecycle::Entering => {
        if let Some(message) = self.window.next() {
          Lifecycle::BeginFrame { message: Some(message) }
        } else {
          Lifecycle::ExitLoop
        }
      }
      Lifecycle::BeginFrame { message } => {
        let message = message.take();
        if let Err(error) = self.game_mailbox.send_and_wait(GameLoopMessage::SyncWithRenderer) {
          error!("{error}");
          Lifecycle::ExitLoop
        } else {
          Lifecycle::EarlyUpdate { message }
        }
      }
      Lifecycle::EarlyUpdate { message } => {
        let message = message.take();
        self.time.update();
        if self.time.should_do_tick() {
          self.time.tick();
          Lifecycle::FixedUpdate { message }
        } else {
          Lifecycle::Update { message }
        }
      },
      Lifecycle::FixedUpdate { message } => {
        let message = message.take();
        if self.time.should_do_tick() {
          self.time.tick();
          Lifecycle::FixedUpdate { message }
        } else {
          Lifecycle::Update { message }
        }
      },
      Lifecycle::Update { message } => {
        let message = message.take();
        Lifecycle::EndFrame { message }
      },
      Lifecycle::EndFrame { message } => {
        let message = message.take();
        match self.game_sync_or_exit(&message) {
          Ok(value) => {
            if value {
              Lifecycle::ExitLoop
            } else if let Some(message) = self.window.next() {
              Lifecycle::BeginFrame { message: Some(message) }
            } else {
              Lifecycle::ExitLoop
            }
          }
          Err(error) => {
            error!("{error}");
            Lifecycle::ExitLoop
          }
        }
      },
      Lifecycle::Exiting => Lifecycle::ExitLoop,
      Lifecycle::ExitLoop => return None,
    };

    self.current_state = new_state;

    Some(&self.current_state)
  }
}

// impl<'a> Iterator for App<'a> {
//   type Item = Lifecycle<'a>;

//   fn next(&mut self) -> Option<Self::Item> {
//     match &self.current_state {
//       Lifecycle::Entering => {
//         if let Some(message) = self.window.next() {
//           self.current_message = message;
//           self.current_state = Lifecycle::BeginFrame { time: &self.time, window: &self.window, message: &self.current_message };
//         } else {
//           self.current_state = Lifecycle::ExitLoop;
//         }
//       },
//       Lifecycle::BeginFrame { time, window, message } => {}
//       Lifecycle::EarlyUpdate { time, window, message } => {}
//       Lifecycle::FixedUpdate { time, window } => {}
//       Lifecycle::Update { time, window, message } => {}
//       Lifecycle::EndFrame { time, window, message } => {
//         if let Some(message) = self.window.next() {
//           self.current_message = message;
//           self.current_state = Lifecycle::BeginFrame { time: &self.time, window: &self.window, message: &self.current_message };
//         } else {
//           self.current_state = Lifecycle::ExitLoop;
//         }
//       }
//       Lifecycle::Exiting => {
//         self.current_state = Lifecycle::ExitLoop;
//       }
//       Lifecycle::ExitLoop => return None,
//     };

//     Some(self.current_state.clone())
//   }
// }
