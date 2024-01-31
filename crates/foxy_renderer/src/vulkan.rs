#![deny(unsafe_op_in_unsafe_fn)]

use foxy_utils::{log::LogErr, types::handle::Handle};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tracing::*;

use self::{
  device::Device,
  error::VulkanError,
  frame_data::FrameData,
  instance::Instance,
  shader::storage::ShaderStore,
  surface::{Surface, SwapchainSupport},
  swapchain::Swapchain,
};
use crate::{
  renderer::RenderBackend,
  vulkan::swapchain::image_format::{ColorSpace, ImageFormat, PresentMode},
};

pub mod device;
pub mod error;
pub mod frame_data;
pub mod instance;
pub mod queue;
pub mod shader;
pub mod surface;
pub mod swapchain;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum ValidationStatus {
  Enabled,
  #[default]
  Disabled,
}

pub struct Vulkan {
  shader_store: ShaderStore,

  frame_index: usize,
  frame_data: Vec<FrameData>,
  swapchain: Handle<Swapchain>,

  device: Handle<Device>,
  surface: Surface,
  instance: Handle<Instance>,
}

impl RenderBackend for Vulkan {
  fn new(
    window: impl HasRawDisplayHandle + HasRawWindowHandle,
    window_size: foxy_utils::types::prelude::Dimensions,
  ) -> Result<Self, crate::error::RendererError>
  where
    Self: Sized,
  {
    trace!("Initializing Vulkan");
    let instance = Handle::new(Instance::new(
      &window,
      if cfg!(debug_assertions) {
        ValidationStatus::Enabled
      } else {
        ValidationStatus::Disabled
      },
    )?);

    let surface = Surface::new(&window, &instance.get())?;
    let device = Handle::new(Device::new(&surface, instance.clone())?);

    let swapchain = Handle::new(Swapchain::new(&instance, &surface, device.clone(), window_size, ImageFormat {
      color_space: ColorSpace::Unorm,
      present_mode: PresentMode::AutoImmediate,
    })?);
    let frame_data = (0..FrameData::FRAME_OVERLAP)
      .map(|_| FrameData::new(&surface, &instance.get(), &device.get()))
      .collect::<Result<Vec<_>, VulkanError>>()?;

    let shader_store = ShaderStore::new(device.clone());

    Ok(Self {
      instance,
      surface,
      device,
      swapchain,
      frame_data,
      frame_index: 0,
      shader_store,
    })
  }

  fn delete(&mut self) {
    trace!("Waiting for GPU to finish...");
    let _ = unsafe { self.device.get().logical().device_wait_idle() }.log_error();

    trace!("Cleaning up Vulkan resources...");

    self.shader_store.delete();

    for frame in &mut self.frame_data {
      frame.delete(&mut self.device);
    }
    self.swapchain.get_mut().delete();

    self.device.get_mut().delete();
    self.surface.delete();
    self.instance.get_mut().delete();
  }

  fn draw(
    &mut self,
    render_time: foxy_utils::time::Time,
  ) -> Result<(), crate::error::RendererError> {
    Ok(())
  }
}

impl Vulkan {
  pub fn instance(&self) -> Handle<Instance> {
    self.instance.clone()
  }

  pub fn surface(&self) -> &Surface {
    &self.surface
  }

  pub fn device(&self) -> Handle<Device> {
    self.device.clone()
  }

  pub fn shaders(&mut self) -> &mut ShaderStore {
    &mut self.shader_store
  }

  pub fn swapchain_support(&self) -> Result<SwapchainSupport, VulkanError> {
    self.surface.swapchain_support(*self.device.get().physical())
  }

  pub fn current_frame(&self) -> Option<&FrameData> {
    self.frame_data.get(self.frame_index)
  }
}
