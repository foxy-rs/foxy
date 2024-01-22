use ezwin::window::Window;
use messaging::Mailbox;
use tracing::*;

use self::render_data::RenderData;

use super::message::{GameLoopMessage, RenderLoopMessage};

pub mod render_data;

pub struct Renderer {
    render_data: RenderData,
}

impl Drop for Renderer {
    fn drop(&mut self) {}
}

impl Renderer {
    pub const RENDER_THREAD_ID: &'static str = "render";
    pub const FRAME_COUNT: u32 = 2;

    pub fn new(window: &Window) -> anyhow::Result<Self> {
        Ok(Self {
            render_data: RenderData::default(),
        })
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn render_loop(
        mut self,
        mut messenger: Mailbox<RenderLoopMessage, GameLoopMessage>,
    ) -> anyhow::Result<()> {
        std::thread::Builder::new()
            .name(Self::RENDER_THREAD_ID.into())
            .spawn(move || -> anyhow::Result<()> {
                trace!("Beginning render");

                loop {
                    if let GameLoopMessage::Exit =
                        messenger.send_and_wait(RenderLoopMessage::SyncWithGame)?
                    {
                        break;
                    }

                    self.render()?;

                    match messenger.send_and_wait(RenderLoopMessage::SyncWithGame)? {
                        GameLoopMessage::Exit => break,
                        GameLoopMessage::RenderData(render_data) => {
                            self.render_data = render_data;
                        }
                        _ => {}
                    }
                }

                trace!("Ending render");

                Ok(())
            })?;

        Ok(())
    }
}
