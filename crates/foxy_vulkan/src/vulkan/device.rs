use ash::vk;

use super::error::VulkanError;

pub struct Device {
  physical: vk::PhysicalDevice,
  // logical: ash::Device,
}

impl Device {
  pub fn new(instance: &ash::Instance) -> Result<Self, VulkanError> {
    let physical = Self::choose_physical(instance)?;

    Ok(Self {
      physical,
      // logical,
    })
  }

  fn choose_physical(instance: &ash::Instance) -> Result<vk::PhysicalDevice, VulkanError> {
    let physical_devices = unsafe { instance.enumerate_physical_devices() }?;
    // physical_devices

    Ok(physical_devices[0])
  }

  // fn new_logical(instance: &ash::Instance) -> Result<ash::Device, VulkanError> {
  //   Ok(  )
  // }
}
