use self::{render_data::RenderData, vulkan::Vulkan};
use super::message::{GameLoopMessage, RenderLoopMessage};
use ezwin::window::Window;
use messaging::Mailbox;
use tracing::*;

pub mod render_data;
mod vulkan;

pub struct Renderer {
  vulkan: Vulkan,
  render_data: RenderData,
}

impl Renderer {
  pub const RENDER_THREAD_ID: &'static str = "render";
  pub const MAX_FRAME_COUNT: u32 = 2;

  pub fn new(window: &Window) -> anyhow::Result<Self> {
    let vulkan = Vulkan::new(window)?;
    Ok(Self {
      vulkan,
      render_data: RenderData::default(),
    })
  }

  pub fn render(&mut self) -> anyhow::Result<()> {
    Ok(())
  }

  pub fn render_loop(mut self, mut messenger: Mailbox<RenderLoopMessage, GameLoopMessage>) -> anyhow::Result<()> {
    std::thread::Builder::new()
      .name(Self::RENDER_THREAD_ID.into())
      .spawn(move || -> anyhow::Result<()> {
        trace!("Beginning render");

        loop {
          if let GameLoopMessage::Exit = messenger.send_and_wait(RenderLoopMessage::SyncWithGame)? {
            break;
          }

          self.render()?;

          if let GameLoopMessage::RenderData(render_data) = messenger.send_and_wait(RenderLoopMessage::SyncWithGame)? {
            self.render_data = render_data;
          }
        }

        trace!("Ending render");

        Ok(())
      })?;

    Ok(())
  }
}
