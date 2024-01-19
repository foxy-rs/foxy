use tracing::*;
use windows::Win32::Foundation::HWND;

mod resources;

pub struct Renderer {
    
}

impl Drop for Renderer {
    fn drop(&mut self) {}
}

impl Renderer {
    pub const FRAME_COUNT: u32 = 2;

    pub fn new(hwnd: HWND, width: i32, height: i32) -> anyhow::Result<Self> {
        Ok(Self {})
    }

    pub fn render(&mut self) -> anyhow::Result<()> {
        // warn!("Rendering");
        Ok(())
    }
}
