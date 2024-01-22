use tracing::*;
use windows::Win32::Foundation::HWND;

use super::message::{GameLoopMessage, RendererMessenger};

mod resources;

pub struct Renderer {}

impl Drop for Renderer {
    fn drop(&mut self) {}
}

impl Renderer {
    pub const RENDER_THREAD_ID: &'static str = "render";
    pub const FRAME_COUNT: u32 = 2;

    pub fn new(hwnd: HWND, width: i32, height: i32) -> anyhow::Result<Self> {
        Ok(Self {})
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        
        Ok(())
    }

    pub fn render_loop(mut self, mut messenger: RendererMessenger) -> anyhow::Result<()> {
        std::thread::Builder::new()
            .name(Self::RENDER_THREAD_ID.into())
            .spawn(move || -> anyhow::Result<()> {
                trace!("Beginning render");

                loop {
                    if let GameLoopMessage::Exit = messenger.sync_and_recieve()? {
                        break;
                    }

                    self.render()?;

                    match messenger.sync_and_recieve()? {
                        GameLoopMessage::Exit => break,
                        GameLoopMessage::RenderData {} => {
                            
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
