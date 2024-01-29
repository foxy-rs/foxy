#![deny(unsafe_op_in_unsafe_fn)]

use std::{
  collections::HashSet,
  ffi::{c_char, CStr},
  sync::Arc,
};

use anyhow::Context;
use ash::{
  extensions::{ext, khr},
  vk,
};
use itertools::Itertools;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle};
use tracing::*;

use self::builder::{DeviceBuilder, MissingWindow, ValidationStatus, VulkanCreateInfo};
use crate::{
  error::{Debug, VulkanError},
  shader::storage::ShaderStore,
  surface::{Surface, SwapchainSupport},
  unsupported_error,
};

pub mod builder;

pub struct Device {
  shader_store: ShaderStore,

  present_queue: vk::Queue,
  graphics_queue: vk::Queue,

  command_pool: vk::CommandPool,

  logical: Arc<ash::Device>,
  physical: vk::PhysicalDevice,

  surface: Surface,

  debug: Debug,
  instance: ash::Instance,
  _entry: ash::Entry,
}

impl Device {
  pub fn delete(&mut self) {
    trace!("Deleting Device");
    unsafe {
      trace!("> Deleting shaders");
      self.shader_store.delete();

      trace!("> Destroying command pool");
      self.logical.destroy_command_pool(self.command_pool, None);

      trace!("> Destroying logical device");
      self.logical.destroy_device(None);

      trace!("> Deleting surface");
      self.surface.delete();
      self.debug.delete();

      trace!("> Destroying instance");
      self.instance.destroy_instance(None);
    }
  }
}

impl Device {
  const DEVICE_EXTENSIONS: &'static [&'static CStr] = &[khr::Swapchain::NAME];
  const INSTANCE_EXTENSIONS: &'static [&'static CStr] = &[
    khr::Surface::NAME,
    #[cfg(debug_assertions)]
    ext::DebugUtils::NAME,
  ];
  const VALIDATION_LAYERS: &'static [&'static CStr] = &[
    #[cfg(debug_assertions)]
    c"VK_LAYER_KHRONOS_validation",
  ];

  pub fn builder() -> DeviceBuilder<MissingWindow> {
    Default::default()
  }

  pub(crate) fn new<W: HasRawDisplayHandle + HasRawWindowHandle>(
    create_info: VulkanCreateInfo<W>,
  ) -> Result<Self, VulkanError> {
    trace!("Initializing Vulkan");
    let display_handle = create_info.window.raw_display_handle();

    let entry = ash::Entry::linked();
    let instance = Self::new_instance(&entry, display_handle, create_info.validation_status)?;
    let debug = Debug::new(&entry, &instance)?;
    let surface = Surface::new(&create_info.window, &entry, &instance)?;
    let physical = Self::pick_physical_device(&surface, &instance)?;
    let (logical, graphics_queue, present_queue) = Self::new_logical_device(&surface, &instance, physical)?;
    let logical = Arc::new(logical);
    let command_pool = Self::create_command_pool(&surface, &instance, &logical, physical)?;
    let shader_store = ShaderStore::new(logical.clone());

    Ok(Self {
      _entry: entry,
      instance,
      debug,
      surface,
      physical,
      logical,
      command_pool,
      graphics_queue,
      present_queue,
      shader_store,
    })
  }

  pub fn instance(&self) -> &ash::Instance {
    &self.instance
  }

  pub fn physical(&self) -> &vk::PhysicalDevice {
    &self.physical
  }

  pub fn logical(&self) -> Arc<ash::Device> {
    self.logical.clone()
  }

  pub fn surface(&self) -> &Surface {
    &self.surface
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

  pub fn shaders(&mut self) -> &mut ShaderStore {
    &mut self.shader_store
  }

  pub fn begin_single_time_commands(&self) -> Result<vk::CommandBuffer, VulkanError> {
    let allocate_info = vk::CommandBufferAllocateInfo {
      level: vk::CommandBufferLevel::PRIMARY,
      command_pool: self.command_pool,
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

    unsafe { self.logical.free_command_buffers(self.command_pool, &[command_buffer]) };

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

  pub fn swapchain_support(&self) -> Result<SwapchainSupport, VulkanError> {
    self.surface.swapchain_support(self.physical)
  }

  fn select_version(entry: &ash::Entry) -> u32 {
    let (variant, major, minor, patch) =
      match unsafe { entry.try_enumerate_instance_version() }.expect("should always return VK_SUCCESS") {
        // Vulkan 1.1+
        Some(version) => {
          let variant = vk::api_version_variant(version);
          let major = vk::api_version_major(version);
          let minor = vk::api_version_minor(version);
          let patch = vk::api_version_patch(version);
          (variant, major, minor, patch)
        }
        // Vulkan 1.0
        None => (0, 1, 0, 0),
      };

    info!("Driver version: Vulkan {major}.{minor}.{patch}.{variant}");

    let selected_version = vk::make_api_version(0, major, minor, 0);

    {
      let variant = vk::api_version_variant(selected_version);
      let major = vk::api_version_major(selected_version);
      let minor = vk::api_version_minor(selected_version);
      let patch = vk::api_version_patch(selected_version);

      info!("Selected version: Vulkan {major}.{minor}.{patch}.{variant}");
    }

    selected_version
  }

  pub fn find_memory_type(&self, type_filter: u32, properties: vk::MemoryPropertyFlags) -> vk::MemoryType {
    let props = unsafe { self.instance.get_physical_device_memory_properties(self.physical) };

    for mem_type in props.memory_types {
      if (type_filter & (1 << mem_type.heap_index)) != 0 && mem_type.property_flags.contains(properties) {
        return mem_type;
      }
    }
    // for i in 0..props.memory_type_count as usize {
    //   if (type_filter & (1 << i)) != 0 &&
    // props.memory_types[i].property_flags.contains(properties) {     return
    // props.memory_types[i];   }
    // }
    error!("Failed to find supported memory type.");
    vk::MemoryType::default()
  }

  fn request_layers_and_extensions(
    entry: &ash::Entry,
    display_handle: RawDisplayHandle,
    validation_status: ValidationStatus,
  ) -> Result<(Vec<*const c_char>, Vec<*const c_char>), VulkanError> {
    let (supported_layers, supported_extensions) = Self::supported(entry)?;
    let supported_layers = supported_layers
      .iter()
      .map(|l| l.layer_name_as_c_str().unwrap())
      .collect_vec();
    let supported_extensions = supported_extensions
      .iter()
      .map(|e| e.extension_name_as_c_str().unwrap())
      .collect_vec();

    debug!("Supported layers:\n{:#?}", supported_layers);
    debug!("Supported instance extensions:\n{:#?}", supported_extensions);

    // Layers ----------------------

    let mut requested_layers = Self::VALIDATION_LAYERS.iter().collect_vec();

    let mut missing_layers = Vec::new();
    for layer in Self::VALIDATION_LAYERS {
      if !supported_layers.contains(layer) {
        missing_layers.push(*layer);
      }
    }

    if !missing_layers.is_empty() {
      return Err(unsupported_error!(
        "not all requested layers are supported on this device:\nMissing: {missing_layers:?}"
      ));
    }

    if validation_status == ValidationStatus::Disabled {
      requested_layers.clear();
    }

    let requested_layers = requested_layers.iter().map(|l| l.as_ptr()).collect_vec();

    // Extensions ------------------

    let mut requested_extensions = ash_window::enumerate_required_extensions(display_handle)?
      .to_vec()
      .iter()
      .map(|c| unsafe { CStr::from_ptr(*c) })
      .collect_vec();
    requested_extensions.extend_from_slice(Self::INSTANCE_EXTENSIONS);

    let mut missing_extensions: Vec<&CStr> = Vec::new();
    for extension in &requested_extensions {
      if !supported_extensions.contains(extension) {
        missing_extensions.push(extension);
      }
    }

    if !missing_extensions.is_empty() {
      return Err(unsupported_error!(
        "not all requested instance extensions are supported on this device:\nMissing: {missing_extensions:?}"
      ));
    }

    let requested_extensions = requested_extensions.iter().map(|l| l.as_ptr()).collect_vec();

    Ok((requested_layers, requested_extensions))
  }

  fn supported(entry: &ash::Entry) -> Result<(Vec<vk::LayerProperties>, Vec<vk::ExtensionProperties>), VulkanError> {
    // let layers: Vec<*const c_char> = Self::VALIDATION_LAYERS.iter().map(|name|
    // name.as_ptr()).collect();
    let layers = unsafe { entry.enumerate_instance_layer_properties() }?;
    let extensions = unsafe { entry.enumerate_instance_extension_properties(None) }?;

    Ok((layers, extensions))
  }

  fn new_instance(
    entry: &ash::Entry,
    display_handle: RawDisplayHandle,
    validation_status: ValidationStatus,
  ) -> Result<ash::Instance, VulkanError> {
    let version = Self::select_version(entry);

    let app_info = vk::ApplicationInfo::default()
      .api_version(version)
      .engine_name(c"Foxy Framework")
      .engine_version(vk::make_api_version(0, 1, 0, 0))
      .application_name(c"Foxy Framework Application")
      .application_version(vk::make_api_version(0, 1, 0, 0));

    let (requested_layers, requested_extensions) =
      Self::request_layers_and_extensions(entry, display_handle, validation_status)?;

    let instance_create_info = vk::InstanceCreateInfo::default()
      .application_info(&app_info)
      .enabled_layer_names(&requested_layers)
      .enabled_extension_names(&requested_extensions);

    let instance = unsafe { entry.create_instance(&instance_create_info, None)? };

    Ok(instance)
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
  ) -> Result<(ash::Device, vk::Queue, vk::Queue), VulkanError> {
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

    let graphics_queue = unsafe { device.get_device_queue(indices.graphics_family, 0) };
    let present_queue = unsafe { device.get_device_queue(indices.present_family, 0) };

    Ok((device, graphics_queue, present_queue))
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
      return Err(unsupported_error!(
        "not all requested device extensions are supported on this device:\nMissing: {missing_extensions:?}"
      ));
    }

    Ok(())
  }

  fn device_features_supported(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
  ) -> Result<(), VulkanError> {
    let mut physical_device_features = vk::PhysicalDeviceFeatures2::default();
    unsafe { instance.get_physical_device_features2(physical_device, &mut physical_device_features) };

    // 1.0 features
    let supported_features = physical_device_features.features;

    macro_rules! supported_feature {
      ($features:tt, $feature:tt) => {{
        if $features.$feature != true.into() {
          return Err(unsupported_error!(
            "not all requested device features are supported on this device: missing {}",
            stringify!($token)
          ));
        }
      }};
    }

    supported_feature!(supported_features, sampler_anisotropy);

    // 1.1 features
    let supported_features = physical_device_features.p_next as *const vk::PhysicalDeviceVulkan11Features;
    if let Some(_supported_features) = unsafe { supported_features.as_ref() } {
      // 1.2 features
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

  fn is_suitable(surface: &Surface, instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> bool {
    let indices = Self::find_queue_families(surface, instance, physical_device);
    let props = unsafe { instance.get_physical_device_properties(physical_device) };
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

  pub fn queue_families(&self) -> Result<QueueFamilyIndices, VulkanError> {
    Self::find_queue_families(&self.surface, &self.instance, self.physical)
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

    Err(unsupported_error!("Failed to find suitable queue families"))
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
}

#[derive(Default)]
pub struct QueueFamilyIndices {
  pub graphics_family: u32,
  pub present_family: u32,
}

impl QueueFamilyIndices {
  // pub fn complete(&self) -> bool { self.graphics_family.is_some() &&
  // self.present_family.is_some() }
}
