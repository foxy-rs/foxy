use ash::{
  extensions::{ext, khr},
  vk,
};
use itertools::Itertools;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle};
use std::{
  ffi::{c_char, CStr},
  mem::ManuallyDrop,
};
use tracing::*;

use crate::vkUnsupported;

use self::{
  builder::{MissingWindow, ValidationStatus, VulkanBuilder, VulkanCreateInfo},
  device::Device,
  error::{Debug, VulkanError},
  surface::Surface,
};

pub mod builder;
pub mod device;
pub mod error;
pub mod surface;

pub struct Vulkan {
  _entry: ash::Entry,

  instance: ManuallyDrop<ash::Instance>,
  debug: ManuallyDrop<Debug>,
  surface: ManuallyDrop<Surface>,
  device: ManuallyDrop<Device>,
}

impl Drop for Vulkan {
  fn drop(&mut self) {
    trace!("Dropping Vulkan");
    unsafe {
      ManuallyDrop::drop(&mut self.device);
      ManuallyDrop::drop(&mut self.surface);
      ManuallyDrop::drop(&mut self.debug);

      self.instance.destroy_instance(None);
      ManuallyDrop::drop(&mut self.instance);
    }
  }
}

impl Vulkan {
  const VALIDATION_LAYERS: &'static [&'static CStr] = &[
    #[cfg(debug_assertions)]
    c"VK_LAYER_KHRONOS_validation",
  ];

  const INSTANCE_EXTENSIONS: &'static [&'static CStr] = &[
    khr::Surface::NAME,
    #[cfg(debug_assertions)]
    ext::DebugUtils::NAME,
  ];

  pub fn builder() -> VulkanBuilder<MissingWindow> {
    Default::default()
  }

  pub(crate) fn new<W: HasRawDisplayHandle + HasRawWindowHandle>(
    create_info: VulkanCreateInfo<W>,
  ) -> Result<Self, VulkanError> {
    trace!("Initializing Vulkan");
    let display_handle = create_info.window.raw_display_handle();

    let entry = ash::Entry::linked();
    let instance = ManuallyDrop::new(Self::new_instance(&entry, display_handle, create_info.validation_status)?);
    let debug = ManuallyDrop::new(Debug::new(&entry, &instance)?);
    let surface = ManuallyDrop::new(Surface::new(&create_info.window, &entry, &instance)?);
    let device = ManuallyDrop::new(Device::new(&surface, &instance)?);

    Ok(Self {
      _entry: entry,
      instance,
      debug,
      surface,
      device,
    })
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
      return Err(vkUnsupported!(
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
      return Err(vkUnsupported!(
        "not all requested instance extensions are supported on this device:\nMissing: {missing_extensions:?}"
      ));
    }

    let requested_extensions = requested_extensions.iter().map(|l| l.as_ptr()).collect_vec();

    Ok((requested_layers, requested_extensions))
  }

  fn supported(entry: &ash::Entry) -> Result<(Vec<vk::LayerProperties>, Vec<vk::ExtensionProperties>), VulkanError> {
    // let layers: Vec<*const c_char> = Self::VALIDATION_LAYERS.iter().map(|name| name.as_ptr()).collect();
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
}
