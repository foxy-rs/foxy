use foxy_utils::types::handle::Handle;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use super::Device;
use crate::error::VulkanError;

pub struct MissingWindow;
pub struct HasWindow<W: HasRawDisplayHandle + HasRawWindowHandle>(W);

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum ValidationStatus {
  Enabled,
  #[default]
  Disabled,
}

pub struct DeviceBuilder<W> {
  window: W,
  validation_status: ValidationStatus,
}

impl Default for DeviceBuilder<MissingWindow> {
  fn default() -> Self {
    Self::new()
  }
}

impl DeviceBuilder<MissingWindow> {
  pub fn new() -> Self {
    Self {
      window: MissingWindow,
      validation_status: Default::default(),
    }
  }
}

impl DeviceBuilder<MissingWindow> {
  pub fn with_window<W: HasRawDisplayHandle + HasRawWindowHandle>(self, window: W) -> DeviceBuilder<HasWindow<W>> {
    DeviceBuilder {
      window: HasWindow(window),
      validation_status: self.validation_status,
    }
  }
}

impl<W> DeviceBuilder<W> {
  pub fn with_validation(self, validation_status: ValidationStatus) -> DeviceBuilder<W> {
    DeviceBuilder {
      window: self.window,
      validation_status,
    }
  }
}

impl<W: HasRawDisplayHandle + HasRawWindowHandle> DeviceBuilder<HasWindow<W>> {
  pub fn build(self) -> Result<Handle<Device>, VulkanError> {
    Ok(Handle::new(Device::new(VulkanCreateInfo {
      window: self.window.0,
      validation_status: self.validation_status,
    })?))
  }
}

pub struct VulkanCreateInfo<W: HasRawDisplayHandle + HasRawWindowHandle> {
  pub window: W,
  pub validation_status: ValidationStatus,
}
