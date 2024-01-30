#![deny(unsafe_op_in_unsafe_fn)]

use ash::vk;
use foxy_utils::{log::LogErr, types::handle::Handle};
use itertools::Itertools;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tracing::*;

use self::{
  builder::{DeviceBuilder, MissingWindow, VulkanCreateInfo},
  device::Device,
  error::VulkanError,
  frame_data::FrameData,
  instance::Instance,
  shader::storage::ShaderStore,
  surface::{Surface, SwapchainSupport},
  swapchain::Swapchain,
};
use crate::{
  vulkan::swapchain::image_format::{ColorSpace, ImageFormat, PresentMode},
  vulkan_error,
};

pub mod builder;
pub mod device;
pub mod error;
pub mod frame_data;
pub mod instance;
pub mod queue;
pub mod shader;
pub mod surface;
pub mod swapchain;
pub mod sync_objects;

pub struct Vulkan {
  shader_store: ShaderStore,

  frame_index: usize,
  frame_data: Vec<FrameData>,
  swapchain: Handle<Swapchain>,

  device: Handle<Device>,
  surface: Surface,
  instance: Handle<Instance>,
}

impl Vulkan {
  pub fn delete(&mut self) {
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
}

impl Vulkan {
  pub fn builder() -> DeviceBuilder<MissingWindow> {
    Default::default()
  }

  pub(crate) fn new<W: HasRawDisplayHandle + HasRawWindowHandle>(
    create_info: VulkanCreateInfo<W>,
  ) -> Result<Self, VulkanError> {
    trace!("Initializing Vulkan");
    let instance = Handle::new(Instance::new(&create_info.window, create_info.validation_status)?);
    let surface = Surface::new(&create_info.window, &instance.get())?;
    let device = Handle::new(Device::new(&surface, instance.clone())?);

    let swapchain = Handle::new(Swapchain::new(
      &instance,
      &surface,
      device.clone(),
      create_info.size,
      ImageFormat {
        color_space: ColorSpace::Unorm,
        present_mode: PresentMode::AutoImmediate,
      },
    )?);
    let frame_data = Self::new_frame_data(&surface, &instance.get(), &device.get())?;

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

  fn new_frame_data(surface: &Surface, instance: &Instance, device: &Device) -> Result<Vec<FrameData>, VulkanError> {
    let create_info = vk::CommandPoolCreateInfo::default()
      .queue_family_index(device.graphics().family())
      .flags(vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    (0..FrameData::FRAME_OVERLAP)
      .map(|_| {
        let command_pool = unsafe { device.logical().create_command_pool(&create_info, None) }?;

        let buffer_info = vk::CommandBufferAllocateInfo::default()
          .command_pool(command_pool)
          .command_buffer_count(1)
          .level(vk::CommandBufferLevel::PRIMARY);

        let master_command_buffer = unsafe { device.logical().allocate_command_buffers(&buffer_info) }?
          .first()
          .cloned()
          .ok_or_else(|| vulkan_error!("invalid command buffers size"))?;

        Ok(FrameData {
          command_pool,
          master_command_buffer,
        })
      })
      .collect::<Result<Vec<_>, VulkanError>>()
  }
}
