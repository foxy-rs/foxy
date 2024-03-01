use std::sync::Arc;

use ezwin::prelude::{Message, Window};
use foxy_time::Time;
use vulkano::{
  instance::{Instance, InstanceCreateInfo},
  swapchain::Surface,
  VulkanLibrary,
};

use self::{device::FoxyDevice, instance::FoxyInstance, render_data::RenderData};
use crate::error::RendererError;

mod debug;
mod device;
mod instance;
pub mod render_data;
mod shader;

// pub struct Egui {
//   context: egui::Context,
//   input: egui::RawInput,
//   state: egui_winit::State,
// }

pub struct Renderer {
  is_dirty: bool,
  instance: FoxyInstance,
  surface: Arc<Surface>,
  device: FoxyDevice,
}

impl Renderer {
  pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
    let instance = FoxyInstance::new(&window)?;
    let surface = Surface::from_window(instance.vk().clone(), window)?;
    let device = FoxyDevice::new(instance.vk().clone(), surface.clone())?;
    Ok(Self {
      is_dirty: false,
      instance,
      surface,
      device,
    })
  }

  pub fn delete(&mut self) {}

  pub fn render(&mut self, render_time: Time, render_data: RenderData) -> Result<(), RendererError> {
    Ok(())
  }

  pub fn resize(&mut self) {}

  pub fn input(&mut self, message: &Message) -> bool {
    false
  }
}
