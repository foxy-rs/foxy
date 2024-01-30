use std::{
  sync::{Arc, Barrier},
  thread::JoinHandle,
};

use anyhow::anyhow;
use foxy_renderer::renderer::Renderer;
use foxy_types::thread::ThreadLoop;
use foxy_util::{log::LogErr, time::EngineTime};
use messaging::Mailbox;
use tracing::*;

use super::message::{GameLoopMessage, RenderLoopMessage};

pub struct RenderLoop {
  pub renderer: Renderer,
  pub messenger: Mailbox<RenderLoopMessage, GameLoopMessage>,
  pub sync_barrier: Arc<Barrier>,
  pub time: EngineTime,
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
          // if self.renderer_sync_or_exit()? {
          //   break;
          // }
          self.sync_barrier.wait();
          self.time.update();
          
          while self.time.should_do_tick_unchecked() {
            self.time.tick();
          }

          if let Err(error) = self.renderer.draw_frame(self.time.time()) {
            error!("{error}");    
          }

          if self.renderer_sync_or_exit()? {
            break;
          }
        }

        trace!("Ending render");

        self.renderer.wait_for_gpu();
        self.renderer.delete();

        Ok(())
      })
      .map_err(anyhow::Error::from)
  }
}

impl RenderLoop {
  fn renderer_sync_or_exit(&mut self) -> anyhow::Result<bool> {
    // self.sync_barrier.wait();
    match self.messenger.send_and_wait(RenderLoopMessage::Response {
      delta_time: *self.time.time().delta(),
      average_delta_time: *self.time.time().average_delta(),
    }) {
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
