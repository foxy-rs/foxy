use ash::{extensions::khr, vk};
use ezwin::raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use super::error::VulkanError;
use crate::{vulkan::instance::Instance, vulkan_error};

pub struct Surface {
  surface: vk::SurfaceKHR,
  surface_loader: khr::Surface,
}

impl Surface {
  pub fn delete(&mut self) {
    unsafe { self.surface_loader.destroy_surface(self.surface, None) };
  }
}

impl Surface {
  pub fn new(window: impl HasRawDisplayHandle + HasRawWindowHandle, instance: &Instance) -> Result<Self, VulkanError> {
    let surface = unsafe {
      ash_window::create_surface(
        instance.entry(),
        instance.raw(),
        window.raw_display_handle(),
        window.raw_window_handle(),
        None,
      )
    }?;

    let surface_loader = khr::Surface::new(instance.entry(), instance.raw());

    Ok(Self {
      surface,
      surface_loader,
    })
  }

  pub fn surface(&self) -> &vk::SurfaceKHR {
    &self.surface
  }

  pub fn surface_loader(&self) -> &khr::Surface {
    &self.surface_loader
  }

  pub fn swapchain_support(&self, physical_device: vk::PhysicalDevice) -> Result<SwapchainSupport, VulkanError> {
    Ok(SwapchainSupport {
      capabilities: unsafe {
        self
          .surface_loader()
          .get_physical_device_surface_capabilities(physical_device, *self.surface())
      }?,
      formats: unsafe {
        self
          .surface_loader()
          .get_physical_device_surface_formats(physical_device, *self.surface())
      }?,
      present_modes: unsafe {
        self
          .surface_loader()
          .get_physical_device_surface_present_modes(physical_device, *self.surface())
      }?,
    })
  }
}

#[derive(Default)]
pub struct SwapchainSupport {
  pub capabilities: vk::SurfaceCapabilitiesKHR,
  pub formats: Vec<vk::SurfaceFormatKHR>,
  pub present_modes: Vec<vk::PresentModeKHR>,
}
