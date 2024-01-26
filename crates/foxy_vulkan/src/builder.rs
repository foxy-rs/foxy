use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{error::VulkanError, vulkan::Vulkan};

pub struct MissingWindow;
pub struct HasWindow<W: HasRawDisplayHandle + HasRawWindowHandle>(W);

pub struct VulkanBuilder<W> {
  window: W,
}

impl Default for VulkanBuilder<MissingWindow> {
  fn default() -> Self {
    Self::new()
  }
}

impl VulkanBuilder<MissingWindow> {
  pub fn new() -> Self {
    Self { window: MissingWindow }
  }
}

impl VulkanBuilder<MissingWindow> {
  pub fn with_window<W: HasRawDisplayHandle + HasRawWindowHandle>(self, window: W) -> VulkanBuilder<HasWindow<W>> {
    VulkanBuilder {
      window: HasWindow(window),
    }
  }
}

impl<W: HasRawDisplayHandle + HasRawWindowHandle> VulkanBuilder<HasWindow<W>> {
  pub fn build(self) -> Result<Vulkan, VulkanError> {
    Vulkan::new(self.window.0)
  }
}
