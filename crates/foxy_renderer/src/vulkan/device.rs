use std::{
  collections::HashSet,
  ffi::{c_void, CStr},
};

use anyhow::Context;
use itertools::Itertools;
use tracing::*;

use super::{
  error::VulkanError,
  image,
  instance::FoxyInstance,
  surface::Surface,
  types::queue::{Queue, QueueFamilyIndices},
};
use crate::vulkan_unsupported_error;

#[derive(Clone)]
pub struct Device {
  instance: FoxyInstance,
  physical: vk::PhysicalDevice,
  logical: ash::Device,
  graphics: Queue,
  present: Queue,
}

impl Device {
  const DEVICE_EXTENSIONS: &'static [&'static CStr] = &[khr::Swapchain::name()];

  pub fn new(surface: &Surface, instance: FoxyInstance) -> Result<Self, VulkanError> {
    let physical = Self::pick_physical_device(surface, &instance)?;
    let (logical, graphics, present) = Self::new_logical_device(surface, &instance, physical)?;

    Ok(Self {
      instance,
      physical,
      logical,
      graphics,
      present,
    })
  }

  pub fn delete(&mut self) {
    unsafe {
      self.logical.destroy_device(None);
    }
  }

  pub fn physical(&self) -> &vk::PhysicalDevice {
    &self.physical
  }

  pub fn logical(&self) -> &ash::Device {
    &self.logical
  }

  pub fn graphics(&self) -> &Queue {
    &self.graphics
  }

  pub fn present(&self) -> &Queue {
    &self.present
  }

  pub fn transition_image(
    &self,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
  ) {
    let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL {
      vk::ImageAspectFlags::DEPTH
    } else {
      vk::ImageAspectFlags::COLOR
    };

    let image_barrier = vk::ImageMemoryBarrier2::builder()
      .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
      .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
      .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
      .dst_access_mask(vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::MEMORY_READ)
      .old_layout(old_layout)
      .new_layout(new_layout)
      .subresource_range(image::image_subresource_range(aspect_mask))
      .image(image);

    let image_barriers = &[*image_barrier];
    let dependency_info = vk::DependencyInfo::builder().image_memory_barriers(image_barriers);

    unsafe { self.logical().cmd_pipeline_barrier2(cmd, &dependency_info) };
  }

  pub fn copy_image(
    &self,
    cmd: vk::CommandBuffer,
    source: vk::Image,
    dest: vk::Image,
    source_size: vk::Extent2D,
    dest_size: vk::Extent2D,
  ) {
    let src_subres = vk::ImageSubresourceLayers::builder()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .base_array_layer(0)
      .layer_count(1)
      .mip_level(0);
    let dst_subres = vk::ImageSubresourceLayers::builder()
      .aspect_mask(vk::ImageAspectFlags::COLOR)
      .base_array_layer(0)
      .layer_count(1)
      .mip_level(0);

    let blit_region = vk::ImageBlit2::builder()
      .src_offsets([vk::Offset3D::default(), vk::Offset3D {
        x: source_size.width as i32,
        y: source_size.height as i32,
        z: 1,
      }])
      .dst_offsets([vk::Offset3D::default(), vk::Offset3D {
        x: dest_size.width as i32,
        y: dest_size.height as i32,
        z: 1,
      }])
      .src_subresource(*src_subres)
      .dst_subresource(*dst_subres);

    let regions = &[*blit_region];
    let blit_info = vk::BlitImageInfo2::builder()
      .dst_image(dest)
      .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
      .src_image(source)
      .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
      .filter(vk::Filter::LINEAR)
      .regions(regions);

    unsafe { self.logical().cmd_blit_image2(cmd, &blit_info) };
  }
}

impl Device {
  #[allow(unused)]
  pub fn find_supported_format(
    &self,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
  ) -> vk::Format {
    for format in candidates.iter() {
      let props = unsafe {
        self
          .instance
          .raw()
          .get_physical_device_format_properties(self.physical, *format)
      };

      if (tiling == vk::ImageTiling::LINEAR && props.linear_tiling_features.contains(features))
        || (tiling == vk::ImageTiling::OPTIMAL && props.optimal_tiling_features.contains(features))
      {
        return *format;
      }
    }
    error!("Failed to find supported format.");
    vk::Format::B8G8R8_UNORM
  }

  pub fn find_memory_type(&self, type_filter: u32, properties: vk::MemoryPropertyFlags) -> vk::MemoryType {
    let props = unsafe { self.instance.raw().get_physical_device_memory_properties(self.physical) };

    for mem_type in props.memory_types {
      if (type_filter & (1 << mem_type.heap_index)) != 0 && mem_type.property_flags.contains(properties) {
        return mem_type;
      }
    }

    error!("Failed to find supported memory type.");
    vk::MemoryType::default()
  }

  fn pick_physical_device(surface: &Surface, instance: &FoxyInstance) -> Result<vk::PhysicalDevice, VulkanError> {
    let physical_devices = unsafe { instance.raw().enumerate_physical_devices() }?;
    info!("Physical device count: {}", physical_devices.len());

    let physical_device = physical_devices
      .iter()
      .filter(|p| Self::is_suitable(surface, instance, **p))
      .min_by_key(|p| unsafe {
        // lower score for preferred device types
        match instance.raw().get_physical_device_properties(**p).device_type {
          vk::PhysicalDeviceType::DISCRETE_GPU => 0,
          vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
          vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
          vk::PhysicalDeviceType::CPU => 3,
          vk::PhysicalDeviceType::OTHER => 4,
          _ => 5,
        }
      })
      .context("Failed to find valid physical device")?;

    let props = unsafe { instance.raw().get_physical_device_properties(*physical_device) };

    let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) };
    info!("Chosen device: [{:?}]", device_name);

    Ok(*physical_device)
  }

  fn new_logical_device(
    surface: &Surface,
    instance: &FoxyInstance,
    physical_device: vk::PhysicalDevice,
  ) -> Result<(ash::Device, Queue, Queue), VulkanError> {
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

    let mut features_13 = vk::PhysicalDeviceVulkan13Features::builder()
      .dynamic_rendering(true)
      .synchronization2(true);

    let mut features_12 = vk::PhysicalDeviceVulkan12Features::builder()
      .buffer_device_address(true)
      .descriptor_indexing(true);
    features_12.p_next = std::ptr::addr_of_mut!(features_13) as *mut c_void;

    let mut features_11 = vk::PhysicalDeviceVulkan11Features {
      p_next: std::ptr::addr_of_mut!(features_12) as *mut c_void,
      ..Default::default()
    };

    let device_features = vk::PhysicalDeviceFeatures::builder().sampler_anisotropy(true);

    let enabled_device_extensions = Self::DEVICE_EXTENSIONS.iter().map(|e| e.as_ptr()).collect_vec();

    let create_info = vk::DeviceCreateInfo::builder()
      .queue_create_infos(&queue_create_infos)
      .enabled_extension_names(&enabled_device_extensions)
      .enabled_features(&device_features)
      .push_next(&mut features_11);

    let device = unsafe { instance.raw().create_device(physical_device, &create_info, None) }
      .context("Failed to create logical graphics device")?;

    let graphics_queue = unsafe { device.get_device_queue(indices.graphics_family, 0) };
    let present_queue = unsafe { device.get_device_queue(indices.present_family, 0) };

    let graphics = Queue::new(graphics_queue, indices.graphics_family);
    let present = Queue::new(present_queue, indices.present_family);

    Ok((device, graphics, present))
  }

  fn device_extensions_supported(
    instance: &FoxyInstance,
    physical_device: vk::PhysicalDevice,
  ) -> Result<(), VulkanError> {
    let supported_extensions = unsafe { instance.raw().enumerate_device_extension_properties(physical_device) }?;
    let supported_extensions = supported_extensions
      .iter()
      .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) })
      .collect_vec();

    debug!("Supported device extensions:\n{:#?}", supported_extensions);

    let mut missing_extensions: Vec<&CStr> = Vec::new();
    for extension in Self::DEVICE_EXTENSIONS {
      if !supported_extensions.contains(extension) {
        missing_extensions.push(extension);
      }
    }

    if !missing_extensions.is_empty() {
      return Err(vulkan_unsupported_error!(
        "not all requested device extensions are supported on this device:\nMissing: {missing_extensions:?}"
      ));
    }

    Ok(())
  }

  fn device_features_supported(
    instance: &FoxyInstance,
    physical_device: vk::PhysicalDevice,
  ) -> Result<(), VulkanError> {
    let mut physical_device_features = vk::PhysicalDeviceFeatures2::default();
    unsafe {
      instance
        .raw()
        .get_physical_device_features2(physical_device, &mut physical_device_features)
    };

    // 1.0 features
    let supported_features = physical_device_features.features;

    macro_rules! supported_feature {
      ($features:tt, $feature:tt) => {{
        if $features.$feature != true.into() {
          return Err(vulkan_unsupported_error!(
            "not all requested device features are supported on this device: missing {}",
            stringify!($feature)
          ));
        }
      }};
    }

    supported_feature!(supported_features, sampler_anisotropy);

    // 1.1 features
    let supported_features = physical_device_features.p_next as *const vk::PhysicalDeviceVulkan11Features;
    if let Some(_supported_features) = unsafe { supported_features.as_ref() } {
      // supported_ext_feature!(supported_features,
      // <vk::PhysicalDeviceShaderClockFeaturesKHR>, shader_clock); 1.2 features
      let supported_features = physical_device_features.p_next as *const vk::PhysicalDeviceVulkan12Features;
      if let Some(supported_features) = unsafe { supported_features.as_ref() } {
        supported_feature!(supported_features, buffer_device_address);
        supported_feature!(supported_features, descriptor_indexing);
        // 1.3 features
        let supported_features = physical_device_features.p_next as *const vk::PhysicalDeviceVulkan13Features;
        if let Some(supported_features) = unsafe { supported_features.as_ref() } {
          supported_feature!(supported_features, dynamic_rendering);
          supported_feature!(supported_features, synchronization2);
        }
      }
    }

    Ok(())
  }

  fn is_suitable(surface: &Surface, instance: &FoxyInstance, physical_device: vk::PhysicalDevice) -> bool {
    let indices = Self::find_queue_families(surface, instance, physical_device);
    let props = unsafe { instance.raw().get_physical_device_properties(physical_device) };
    let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) };

    debug!("Checking if suitable: [{:?}]", device_name);
    // debug!("Checking if suitable: [{}]", unsafe {
    // std::str::from_utf8_unchecked(std::mem::transmute(&props.device_name as
    // &[i8])) });

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

    let features_supported = match Self::device_features_supported(instance, physical_device) {
      Ok(_) => true,
      Err(e) => {
        error!("{e}");
        false
      }
    };

    indices.is_ok() && extensions_supported && swapchain_adequate && features_supported
  }

  fn find_queue_families(
    surface: &Surface,
    instance: &FoxyInstance,
    physical_device: vk::PhysicalDevice,
  ) -> Result<QueueFamilyIndices, VulkanError> {
    let queue_families = unsafe {
      instance
        .raw()
        .get_physical_device_queue_family_properties(physical_device)
    };

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

    Err(vulkan_unsupported_error!("Failed to find suitable queue families"))
  }
}
