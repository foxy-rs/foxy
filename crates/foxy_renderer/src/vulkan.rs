#![deny(unsafe_op_in_unsafe_fn)]

use std::sync::Arc;

use foxy_utils::time::Time;
use tracing::*;
use vulkano::{command_buffer::allocator::StandardCommandBufferAllocator, memory::allocator::StandardMemoryAllocator};
use winit::{event::WindowEvent, window::Window};

use self::{instance::FoxyInstance, surface::FoxySurface};
use crate::{
  error::RendererError,
  renderer::render_data::RenderData,
  vulkan::{
    device::FoxyDevice,
    swapchain::{image_format::PresentMode, FoxySwapchain},
  },
};

mod device;
pub mod error;
mod instance;
mod surface;
mod swapchain;

pub struct Vulkan {
  window: Arc<Window>,
  instance: FoxyInstance,
  surface: FoxySurface,
  device: FoxyDevice,
  swapchain: FoxySwapchain,
  allocator: Arc<StandardMemoryAllocator>,
  cmd_buffer_allocator: Arc<StandardCommandBufferAllocator>,
}

impl Vulkan {
  pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
    trace!("Initializing Vulkan");

    let instance = FoxyInstance::new(&window)?;
    let surface = FoxySurface::new(instance.vk().clone(), window.clone())?;
    let device = FoxyDevice::new(instance.vk().clone(), surface.vk().clone())?;
    let swapchain = FoxySwapchain::new(
      surface.vk().clone(),
      device.vk().clone(),
      window.inner_size(),
      PresentMode::AutoImmediate,
    )?;

    let allocator = Arc::new(StandardMemoryAllocator::new_default(device.vk().clone()));

    let cmd_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(device.vk().clone(), Default::default()));

    Ok(Self {
      window,
      instance,
      surface,
      device,
      swapchain,
      allocator,
      cmd_buffer_allocator,
    })
  }

  pub fn render(&mut self, render_time: Time, _render_data: RenderData) -> Result<bool, RendererError> {
    Ok(false)
  }

  pub fn resize(&mut self) {}

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    false
  }
}

impl Vulkan {}
