use std::sync::mpsc::{Receiver, Sender};

use tracing::*;
use windows::Win32::Foundation::HWND;

use super::message::{GameLoopMessage, RenderLoopMessage};

mod resources;

pub struct Renderer {
    
}

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
        // warn!("Rendering");
        Ok(())
    }

    pub fn render_loop(mut self, sender: Sender<RenderLoopMessage>, reciever: Receiver<GameLoopMessage>) -> anyhow::Result<()> {
        std::thread::Builder::new()
            .name(Self::RENDER_THREAD_ID.into())
            .spawn(move || -> anyhow::Result<()> {
                trace!("Beginning render");

                loop {
                    sender.send(RenderLoopMessage::SyncWithGame)?;
                    match reciever.recv()? {
                        GameLoopMessage::Exit => break,
                        GameLoopMessage::SyncWithRenderer => {
                            // trace!("PRE: Render synced!");
                        },
                        _ => {},
                    }
    
                    self.render()?;
    
                    sender.send(RenderLoopMessage::SyncWithGame)?;
                    match reciever.recv()? {
                        GameLoopMessage::Exit => break,
                        GameLoopMessage::SyncWithRenderer => {
                            // trace!("POST: Render synced!");
                        },
                        _ => {},
                    }

                    match reciever.recv()? {
                        GameLoopMessage::Exit => break,
                        GameLoopMessage::RenderData { .. } => {
                            // trace!("Copying game thread data to renderer");
                        }
                        _ => {},
                    }
                }

                trace!("Ending render");

                Ok(())
            })?;

        Ok(())
    }
}
