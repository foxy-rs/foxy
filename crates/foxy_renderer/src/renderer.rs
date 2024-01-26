use self::render_data::RenderData;
use foxy_vulkan::{builder::ValidationStatus, vulkan::Vulkan};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub mod message;
pub mod render_data;

pub struct Renderer {
  vulkan: Vulkan,
  render_data: RenderData,
}

impl Renderer {
  pub const MAX_FRAME_COUNT: u32 = 2;

  pub fn new(window: impl HasRawDisplayHandle + HasRawWindowHandle) -> anyhow::Result<Self> {
    let vulkan = Vulkan::builder()
      .with_window(&window)
      .with_validation(ValidationStatus::Enabled)
      .build()?;

    Ok(Self {
      vulkan,
      render_data: RenderData::default(),
    })
  }

  pub fn render(&mut self) -> anyhow::Result<()> {
    Ok(())
  }

  pub fn update_render_data(&mut self, render_data: RenderData) -> anyhow::Result<()> {
    Ok(())
  }
}
