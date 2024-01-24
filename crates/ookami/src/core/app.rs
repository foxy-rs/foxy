use std::thread::JoinHandle;

use self::{
  builder::{AppCreateInfo, HasSize, HasTitle},
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

struct App<State> {
  time: Time,
  lifecycle: State,
  // These are optional to allow take-ing in main loop
  window: Option<Window>,
  renderer: Option<Renderer>,
  render_thread: Option<JoinHandle<anyhow::Result<()>>>,
}

impl<Life: Lifecycle> App<Life> {
  pub const RENDER_THREAD_ID: &'static str = "render";

  pub fn new(app_create_info: AppCreateInfo<HasTitle, HasSize>, lifecycle: Life) -> anyhow::Result<Self> {
    let time = Time::new(128.0, 1024);
    let mut window = WindowBuilder::new()
      .with_title(app_create_info.title.0)
      .with_size(app_create_info.size.width, app_create_info.size.height)
      .with_color_mode(app_create_info.color_mode)
      .with_close_behavior(app_create_info.close_behavior)
      .with_visibility(Visibility::Hidden)
      .build()?;
    let renderer = Renderer::new(&window)?;
    window.set_visibility(Visibility::Shown);

    Ok(Self {
      time,
      lifecycle,
      window: Some(window),
      renderer: Some(renderer),
      render_thread: None
    })
  }

  fn run_internal(mut self) -> anyhow::Result<()> {
    let (renderer_mailbox, game_mailbox) = Mailbox::new_entangled_pair();

    // to allow double mutable borrow
    if let (Some(mut window), Some(renderer)) = (self.window.take(), self.renderer.take()) {
      self.render_thread = Some(Self::render_loop(renderer, renderer_mailbox)?);
      self.game_loop(&mut window, game_mailbox)?;
    };

    Ok(())
  }

  fn game_loop(
    &mut self,
    window: &mut Window,
    mut messenger: Mailbox<GameLoopMessage, RenderLoopMessage>,
  ) -> anyhow::Result<()> {
    self.lifecycle.start(&self.time, window);

    while let Some(message) = window.next() {
      match message {
        WindowMessage::Closed => {
          messenger.send_and_wait(GameLoopMessage::Exit)?;
          if let Err(error) = self.render_thread.take().expect("render_thread handle should not be None").join() {
            error!("{error:?}");
          }
          break;
        }
        _ => {
          messenger.send_and_wait(GameLoopMessage::SyncWithRenderer)?;
        }
      }

      // Main lifecycle
      self.time.update();
      self.lifecycle.early_update(&self.time, window, &message);
      while self.time.should_do_tick() {
        self.time.tick();
        self.lifecycle.fixed_update(&self.time, window);
      }
      self.lifecycle.update(&self.time, window, &message);

      messenger.send_and_wait(GameLoopMessage::RenderData(RenderData {}))?;
    }

    self.lifecycle.stop(&self.time, window);

    Ok(())
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
          if let GameLoopMessage::Exit = messenger.send_and_wait(RenderLoopMessage::SyncWithGame)? {
            break;
          }

          renderer.render()?;

          if let GameLoopMessage::RenderData(render_data) = messenger.send_and_wait(RenderLoopMessage::SyncWithGame)? {
            renderer.update(render_data)?;
          }
        }

        trace!("Ending render");

        Ok(())
      }).map_err(anyhow::Error::from)
  }

  pub fn run(self) {
    trace!("uchi uchi, uchi da yo");
    if let Err(error) = self.run_internal() {
      error!("{error}");
    }
    trace!("otsu mion");
  }
}
