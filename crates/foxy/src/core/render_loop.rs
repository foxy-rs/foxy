use std::{
  sync::{Arc, Barrier},
  thread::JoinHandle,
};

use anyhow::anyhow;
use foxy_renderer::renderer::Renderer;
use messaging::Mailbox;
use tracing::*;

use super::message::{GameLoopMessage, RenderLoopMessage};

pub struct RenderThread {
  join_handle: Option<JoinHandle<anyhow::Result<()>>>,
  render_loop: Option<RenderLoop>,
}

impl RenderThread {
  pub fn new(
    renderer: Renderer,
    messenger: Mailbox<RenderLoopMessage, GameLoopMessage>,
    sync_barrier: Arc<Barrier>,
  ) -> Self {
    Self {
      join_handle: None,
      render_loop: Some(RenderLoop {
        renderer,
        messenger,
        sync_barrier,
      }),
    }
  }

  pub fn run(&mut self) {
    if let Some(render_loop) = self.render_loop.take() {
      self.join_handle = render_loop.run().inspect_err(|e| error!("{e}")).ok();
    }
  }

  pub fn join(&mut self) {
    if let Some(join_handle) = self.join_handle.take() {
      if let Err(error) = join_handle.join() {
        error!("{error:?}");
      } else {
        trace!("render thread joined");
      }
    } else {
      error!("render thread join_handle was None!");
    }
  }
}

pub struct RenderLoop {
  renderer: Renderer,
  messenger: Mailbox<RenderLoopMessage, GameLoopMessage>,
  sync_barrier: Arc<Barrier>,
}

impl RenderLoop {
  pub const RENDER_THREAD_ID: &'static str = "render";

  pub fn run(mut self) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    std::thread::Builder::new()
      .name(Self::RENDER_THREAD_ID.into())
      .spawn(move || -> anyhow::Result<()> {
        trace!("Beginning render");

        loop {
          if self.renderer_sync_or_exit()? {
            break;
          }

          self.renderer.render()?;

          if self.renderer_sync_or_exit()? {
            break;
          }
        }

        trace!("Ending render");

        Ok(())
      })
      .map_err(anyhow::Error::from)
  }

  fn renderer_sync_or_exit(&mut self) -> anyhow::Result<bool> {
    self.sync_barrier.wait();
    match self.messenger.poll() {
      Ok(message) => match message {
        GameLoopMessage::RenderData(data) => {
          self.renderer.update_render_data(data)?;
          Ok(false)
        }
        GameLoopMessage::Exit => Ok(true),
      },
      Err(error) => {
        if let messaging::MessagingError::PollError {
          error: std::sync::mpsc::TryRecvError::Disconnected,
        } = error
        {
          Err(anyhow!(std::sync::mpsc::TryRecvError::Disconnected))
        } else {
          Ok(false)
        }
      }
    }
  }
}
