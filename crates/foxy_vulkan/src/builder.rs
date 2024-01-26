use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use crate::{error::VulkanError, vulkan::Vulkan};

pub struct MissingWindow;
pub struct HasWindow<W: HasRawDisplayHandle + HasRawWindowHandle>(W);

#[derive(Default, PartialEq, Eq)]
pub enum ValidationStatus {
  Enabled,
  #[default]
  Disabled,
}

pub struct VulkanBuilder<W> {
  window: W,
  validation_status: ValidationStatus,
}

impl Default for VulkanBuilder<MissingWindow> {
  fn default() -> Self {
    Self::new()
  }
}

impl VulkanBuilder<MissingWindow> {
  pub fn new() -> Self {
    Self {
      window: MissingWindow,
      validation_status: Default::default(),
    }
  }
}

impl VulkanBuilder<MissingWindow> {
  pub fn with_window<W: HasRawDisplayHandle + HasRawWindowHandle>(self, window: W) -> VulkanBuilder<HasWindow<W>> {
    VulkanBuilder {
      window: HasWindow(window),
      validation_status: self.validation_status,
    }
  }
}

impl<W> VulkanBuilder<W> {
  pub fn with_validation(self, validation_status: ValidationStatus) -> VulkanBuilder<W> {
    VulkanBuilder {
      window: self.window,
      validation_status,
    }
  }
}

impl<W: HasRawDisplayHandle + HasRawWindowHandle> VulkanBuilder<HasWindow<W>> {
  pub fn build(self) -> Result<Vulkan, VulkanError> {
    Vulkan::new(VulkanCreateInfo {
      window: self.window.0,
      validation_status: self.validation_status,
    })
  }
}

pub struct VulkanCreateInfo<W: HasRawDisplayHandle + HasRawWindowHandle> {
  pub window: W,
  pub validation_status: ValidationStatus,
}
