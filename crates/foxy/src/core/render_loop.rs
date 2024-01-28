use std::{
  sync::{Arc, Barrier},
  thread::JoinHandle,
};

use anyhow::anyhow;
use foxy_renderer::renderer::Renderer;
use foxy_types::thread::ThreadLoop;
use messaging::Mailbox;
use tracing::*;

use super::message::{GameLoopMessage, RenderLoopMessage};

pub struct RenderLoop {
  pub renderer: Renderer,
  pub messenger: Mailbox<RenderLoopMessage, GameLoopMessage>,
  pub sync_barrier: Arc<Barrier>,
}

impl ThreadLoop for RenderLoop {
  type Params = ();

  const THREAD_ID: &'static str = "render";

  fn run(mut self, _: Self::Params) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    std::thread::Builder::new()
      .name(Self::THREAD_ID.into())
      .spawn(move || -> anyhow::Result<()> {
        trace!("Beginning render");

        loop {
          if self.renderer_sync_or_exit()? {
            break;
          }

          self.renderer.draw_frame()?;

          if self.renderer_sync_or_exit()? {
            break;
          }
        }

        trace!("Ending render");

        Ok(())
      })
      .map_err(anyhow::Error::from)
  }
}

impl RenderLoop {
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
