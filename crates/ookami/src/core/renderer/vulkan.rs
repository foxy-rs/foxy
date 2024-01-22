use ash::{prelude::*, vk};
use ezwin::window::Window;
use tracing::*;

pub struct Vulkan {
  instance: vk::Instance,
}

impl Vulkan {
  pub fn new(window: &Window) -> anyhow::Result<Self> {
    let instance = Self::create_instance()?;
    Ok(Self { instance })
  }

  fn create_instance() -> anyhow::Result<vk::Instance> {
    trace!("Creating Vulkan instance");

    let entry = ash::Entry::linked();
    // info!("Vulkan {}.{}.{}");
    
    Ok(vk::Instance::null())
  }
}
