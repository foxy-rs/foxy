use std::{
  collections::HashSet,
  ffi::CStr,
  mem::ManuallyDrop,
  sync::Arc,
};

use anyhow::Context;
use ash::{extensions::khr, vk};
use itertools::Itertools;
use tracing::*;

use crate::vkUnsupported;

use super::{error::VulkanError, surface::Surface};

pub struct Device {
  physical: ManuallyDrop<vk::PhysicalDevice>,
  logical: ManuallyDrop<ash::Device>,
  command_pool: ManuallyDrop<vk::CommandPool>,
  graphics_queue: vk::Queue,
  present_queue: vk::Queue,
}

impl Drop for Device {
  fn drop(&mut self) {
    unsafe {
      self.logical.destroy_command_pool(*self.command_pool, None);
      ManuallyDrop::drop(&mut self.command_pool);

      self.logical.destroy_device(None);
      ManuallyDrop::drop(&mut self.logical);
      ManuallyDrop::drop(&mut self.physical);
    }
  }
}

impl Device {
  const DEVICE_EXTENSIONS: &'static [&'static CStr] = &[khr::Swapchain::NAME];

  pub fn new(surface: &Surface, instance: &ash::Instance) -> Result<Self, VulkanError> {
    let physical = ManuallyDrop::new(Self::pick_physical_device(surface, instance)?);
    let logical = ManuallyDrop::new(Self::new_logical_device(surface, instance, *physical)?);
    let command_pool = ManuallyDrop::new(Self::create_command_pool(surface, instance, &logical, *physical)?);

    Ok(Self {
      physical,
      logical,
      command_pool,
      graphics_queue: Default::default(),
      present_queue: Default::default(),
    })
  }

  fn pick_physical_device(surface: &Surface, instance: &ash::Instance) -> Result<vk::PhysicalDevice, VulkanError> {
    let physical_devices = unsafe { instance.enumerate_physical_devices() }?;
    info!("Physical device count: {}", physical_devices.len());

    let physical_device = physical_devices
      .iter()
      .filter(|p| Self::is_suitable(surface, instance, **p))
      .min_by_key(|p| unsafe {
        // lower score for preferred device types
        match instance.get_physical_device_properties(**p).device_type {
          vk::PhysicalDeviceType::DISCRETE_GPU => 0,
          vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
          vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
          vk::PhysicalDeviceType::CPU => 3,
          vk::PhysicalDeviceType::OTHER => 4,
          _ => 5,
        }
      })
      .context("Failed to find valid physical device")?;

    let props = unsafe { instance.get_physical_device_properties(*physical_device) };

    let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) };
    info!("Chosen device: [{:?}]", device_name);

    Ok(*physical_device)
  }

  fn new_logical_device(
    surface: &Surface,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
  ) -> Result<ash::Device, VulkanError> {
    let indices = Self::find_queue_families(surface, instance, physical_device)?;
    let mut queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = vec![];
    let unique_queue_families: HashSet<u32> = HashSet::from([indices.graphics_family, indices.present_family]);

    let queue_priority = 1.0;
    for queue_family in unique_queue_families {
      let queue_create_info = vk::DeviceQueueCreateInfo {
        queue_family_index: queue_family,
        queue_count: 1,
        p_queue_priorities: &queue_priority,
        ..Default::default()
      };
      queue_create_infos.push(queue_create_info);
    }

    let device_features = vk::PhysicalDeviceFeatures {
      sampler_anisotropy: vk::TRUE,
      ..Default::default()
    };

    let enabled_device_extensions = Self::DEVICE_EXTENSIONS.iter().map(|e| e.as_ptr()).collect_vec();

    let create_info = vk::DeviceCreateInfo {
      queue_create_info_count: queue_create_infos.len() as u32,
      p_queue_create_infos: queue_create_infos.as_ptr(),
      p_enabled_features: &device_features,
      enabled_extension_count: enabled_device_extensions.len() as u32,
      pp_enabled_extension_names: enabled_device_extensions.as_ptr(),
      ..Default::default()
    };

    let device = unsafe { instance.create_device(physical_device, &create_info, None) }
      .context("Failed to create logical graphics device")?;

    Ok(device)
  }

  fn device_extensions_supported(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
  ) -> Result<(), VulkanError> {
    let supported_extensions = unsafe { instance.enumerate_device_extension_properties(physical_device) }?;
    let supported_extensions = supported_extensions
      .iter()
      .map(|e| e.extension_name_as_c_str().unwrap())
      .collect_vec();

    debug!("Supported device extensions:\n{:#?}", supported_extensions);

    let mut missing_extensions: Vec<&CStr> = Vec::new();
    for extension in Self::DEVICE_EXTENSIONS {
      if !supported_extensions.contains(extension) {
        missing_extensions.push(extension);
      }
    }

    if !missing_extensions.is_empty() {
      return Err(vkUnsupported!(
        "not all requested device extensions are supported on this device:\nMissing: {missing_extensions:?}"
      ));
    }

    Ok(())
  }

  fn is_suitable(surface: &Surface, instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> bool {
    let indices = Self::find_queue_families(surface, instance, physical_device);
    let props = unsafe { instance.get_physical_device_properties(physical_device) };
    let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) };

    debug!("Checking if suitable: [{:?}]", device_name);
    // debug!("Checking if suitable: [{}]", unsafe { std::str::from_utf8_unchecked(std::mem::transmute(&props.device_name as &[i8])) });

    let extensions_supported = match Self::device_extensions_supported(instance, physical_device) {
      Ok(_) => true,
      Err(e) => {
        error!("{e}");
        false
      }
    };

    let swapchain_adequate = if extensions_supported {
      let swapchain_support = match surface.swapchain_support(physical_device) {
        Ok(value) => value,
        Err(e) => {
          error!("{e}");
          return false;
        }
      };
      !swapchain_support.formats.is_empty() && !swapchain_support.present_modes.is_empty()
    } else {
      false
    };

    let supported_features = unsafe { instance.get_physical_device_features(physical_device) };

    indices.is_ok() && extensions_supported && swapchain_adequate && supported_features.sampler_anisotropy == vk::TRUE
  }

  #[allow(unused)]
  fn find_supported_format(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    candidates: Arc<[vk::Format]>,
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
  ) -> vk::Format {
    for format in candidates.iter() {
      let props = unsafe { instance.get_physical_device_format_properties(physical_device, *format) };

      if (tiling == vk::ImageTiling::LINEAR && props.linear_tiling_features.contains(features))
        || (tiling == vk::ImageTiling::OPTIMAL && props.optimal_tiling_features.contains(features))
      {
        return *format;
      }
    }
    error!("Failed to find supported format.");
    vk::Format::B8G8R8_UNORM
  }

  fn find_queue_families(
    surface: &Surface,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
  ) -> Result<QueueFamilyIndices, VulkanError> {
    let queue_families = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut graphics_family = None;
    let mut present_family = None;
    for (i, family) in queue_families.iter().enumerate() {
      if family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
        graphics_family = Some(i as u32);
      }

      let present_support = unsafe {
        surface
          .surface_loader()
          .get_physical_device_surface_support(physical_device, i as u32, *surface.surface())
      }?;

      if family.queue_count > 0 && present_support {
        present_family = Some(i as u32);
      }

      if let (Some(graphics_family), Some(present_family)) = (graphics_family, present_family) {
        return Ok(QueueFamilyIndices {
          graphics_family,
          present_family,
        });
      }
    }

    Err(vkUnsupported!("Failed to find suitable queue families"))
  }

  fn create_command_pool(
    surface: &Surface,
    instance: &ash::Instance,
    logical: &ash::Device,
    physical: vk::PhysicalDevice,
  ) -> Result<vk::CommandPool, VulkanError> {
    let indices = Self::find_queue_families(surface, instance, physical)?;

    let create_info = vk::CommandPoolCreateInfo {
      queue_family_index: indices.graphics_family,
      flags: vk::CommandPoolCreateFlags::TRANSIENT | vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
      ..Default::default()
    };

    unsafe { logical.create_command_pool(&create_info, None) }.map_err(VulkanError::from)
  }

  pub fn physical(&self) -> &vk::PhysicalDevice {
    &self.physical
  }

  pub fn logical(&self) -> &ash::Device {
    &self.logical
  }

  pub fn command_pool(&self) -> &vk::CommandPool {
    &self.command_pool
  }

  pub fn graphics_queue(&self) -> &vk::Queue {
    &self.graphics_queue
  }

  pub fn present_queue(&self) -> &vk::Queue {
    &self.present_queue
  }

  pub fn begin_single_time_commands(&self) -> Result<vk::CommandBuffer, VulkanError> {
    let allocate_info = vk::CommandBufferAllocateInfo {
      level: vk::CommandBufferLevel::PRIMARY,
      command_pool: *self.command_pool,
      command_buffer_count: 1,
      ..Default::default()
    };

    let command_buffer =
      unsafe { self.logical.allocate_command_buffers(&allocate_info) }.context("Failed to allocate command buffers")?;

    let begin_info = vk::CommandBufferBeginInfo {
      flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
      ..Default::default()
    };

    unsafe { self.logical.begin_command_buffer(command_buffer[0], &begin_info) }
      .context("Failed to begin command buffer")?;

    Ok(command_buffer[0])
  }

  pub fn end_single_time_commands(&self, command_buffer: vk::CommandBuffer) -> Result<(), VulkanError> {
    unsafe { self.logical.end_command_buffer(command_buffer) }.context("Failed to end command buffer")?;

    let submit_info = vk::SubmitInfo {
      command_buffer_count: 1,
      p_command_buffers: &command_buffer,
      ..Default::default()
    };

    unsafe {
      self
        .logical
        .queue_submit(self.graphics_queue, &[submit_info], vk::Fence::null())
    }
    .context("Failed to submit graphics queue")?;

    unsafe { self.logical.queue_wait_idle(self.graphics_queue) }.context("Failed to process graphics queue")?;

    unsafe { self.logical.free_command_buffers(*self.command_pool, &[command_buffer]) };

    Ok(())
  }

  pub fn issue_single_time_commands<F: FnOnce(vk::CommandBuffer)>(&self, commands: F) {
    match self.begin_single_time_commands() {
      Ok(command_buffer) => {
        commands(command_buffer);
        match self.end_single_time_commands(command_buffer) {
          Ok(_) => {}
          Err(e) => error!("{e:#}"),
        };
      }
      Err(e) => error!("{e:#}"),
    }
  }
}

#[derive(Default)]
struct QueueFamilyIndices {
  pub graphics_family: u32,
  pub present_family: u32,
}

impl QueueFamilyIndices {
  // pub fn complete(&self) -> bool { self.graphics_family.is_some() && self.present_family.is_some() }
}
