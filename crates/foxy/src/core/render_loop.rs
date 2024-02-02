use anyhow::anyhow;
use foxy_renderer::{renderer::Renderer, vulkan::Vulkan};
use foxy_utils::{
  thread::{
    error::ThreadError,
    handle::{HandlesResult, ThreadLoop},
  },
  time::EngineTime,
};
use messaging::Mailbox;
use tracing::*;

use super::message::{GameLoopMessage, RenderLoopMessage};

pub struct RenderLoop {
  pub renderer: Renderer<Vulkan>,
  pub messenger: Mailbox<RenderLoopMessage, GameLoopMessage>,
  pub time: EngineTime,
  pub should_exit: bool,
}

impl ThreadLoop for RenderLoop {
  type Params = ();

  fn run(mut self, thread_id: Vec<String>, _: Self::Params) -> HandlesResult {
    Ok(vec![std::thread::Builder::new()
      .name(thread_id.first().cloned().expect("invalid index"))
      .spawn(move || -> Result<(), ThreadError> {
        trace!("Beginning render");

        loop {
          if self.renderer_sync_or_exit()? {
            break;
          }

          // self.sync_barrier.wait();
          self.time.update();

          while self.time.should_do_tick_unchecked() {
            self.time.tick();
          }

          if let Err(error) = self.renderer.draw_frame(self.time.time()) {
            error!("`{error}` Aborting...");
            let _ = self
              .messenger
              .send_and_wait(RenderLoopMessage::EmergencyExit)
              .map_err(anyhow::Error::from)?;
            break;
          }

          if self.renderer_sync_or_exit()? {
            break;
          }
        }

        trace!("Ending render");

        self.renderer.delete();

        Ok(())
      })
      .map_err(ThreadError::from)?])
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
        GameLoopMessage::Exit => Ok(true),
        GameLoopMessage::RenderInfo {} => Ok(false),
        _ => Ok(false),
      },
      Err(error) => {
        if let messaging::MessagingError::PollError {
          error: std::sync::mpsc::TryRecvError::Disconnected,
        } = error
        {
          Err(anyhow!("renderer_sync_or_exit: {error:?}"))
        } else {
          Ok(false)
        }
      }
    }
  }
}
