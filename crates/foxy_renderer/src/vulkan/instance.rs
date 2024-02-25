use std::ffi::{c_char, CStr};

use ash::{
  extensions::{ext, khr},
  vk,
};
use ezwin::{prelude::Window, raw_window_handle::HasRawDisplayHandle};
use itertools::Itertools;
use tracing::*;

use super::ValidationStatus;
use crate::{
  vulkan::error::{Debug, VulkanError},
  vulkan_unsupported_error,
};

#[derive(Clone)]
pub struct Instance {
  debug: Debug,
  instance: ash::Instance,
  entry: ash::Entry,
}

impl Instance {
  const INSTANCE_EXTENSIONS: &'static [&'static CStr] = &[
    khr::Surface::name(),
    #[cfg(debug_assertions)]
    ext::DebugUtils::name(),
  ];
  const VALIDATION_LAYERS: &'static [&'static CStr] = &[
    #[cfg(debug_assertions)]
    c"VK_LAYER_KHRONOS_validation",
  ];

  pub fn new(window: &Window, validation_status: ValidationStatus) -> Result<Self, VulkanError> {
    let entry = ash::Entry::linked();
    let instance = Self::new_instance(&entry, window, validation_status)?;
    let debug = Debug::new(&entry, &instance)?;

    Ok(Self { debug, instance, entry })
  }

  pub fn delete(&mut self) {
    unsafe {
      self.debug.delete();
      self.instance.destroy_instance(None);
    }
  }

  pub fn entry(&self) -> &ash::Entry {
    &self.entry
  }

  pub fn raw(&self) -> &ash::Instance {
    &self.instance
  }

  fn new_instance(
    entry: &ash::Entry,
    window: &Window,
    validation_status: ValidationStatus,
  ) -> Result<ash::Instance, VulkanError> {
    let version = Self::select_version(entry);

    let app_info = vk::ApplicationInfo::builder()
      .api_version(version)
      .engine_name(c"Foxy Framework")
      .engine_version(vk::make_api_version(0, 1, 0, 0))
      .application_name(c"Foxy Framework Application")
      .application_version(vk::make_api_version(0, 1, 0, 0));

    let (requested_layers, requested_extensions) =
      Self::request_layers_and_extensions(entry, window, validation_status)?;

    let instance_create_info = vk::InstanceCreateInfo::builder()
      .application_info(&app_info)
      .enabled_layer_names(&requested_layers)
      .enabled_extension_names(&requested_extensions);

    let instance = unsafe { entry.create_instance(&instance_create_info, None)? };

    Ok(instance)
  }

  fn select_version(entry: &ash::Entry) -> u32 {
    let (variant, major, minor, patch) = match entry
      .try_enumerate_instance_version()
      .expect("should always return VK_SUCCESS")
    {
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
    window: &Window,
    validation_status: ValidationStatus,
  ) -> Result<(Vec<*const c_char>, Vec<*const c_char>), VulkanError> {
    let (supported_layers, supported_extensions) = Self::supported(entry)?;
    let supported_layers = supported_layers
      .iter()
      .map(|l| unsafe { CStr::from_ptr(l.layer_name.as_ptr()) })
      .collect_vec();
    let supported_extensions = supported_extensions
      .iter()
      .map(|e| unsafe { CStr::from_ptr(e.extension_name.as_ptr()) })
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
      return Err(vulkan_unsupported_error!(
        "not all requested layers are supported on this device:\nMissing: {missing_layers:?}"
      ));
    }

    if validation_status == ValidationStatus::Disabled {
      requested_layers.clear();
    }

    let requested_layers = requested_layers.iter().map(|l| l.as_ptr()).collect_vec();

    // Extensions ------------------

    let mut requested_extensions = ash_window::enumerate_required_extensions(window.raw_display_handle())?
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
      return Err(vulkan_unsupported_error!(
        "not all requested instance extensions are supported on this device:\nMissing: {missing_extensions:?}"
      ));
    }

    let requested_extensions = requested_extensions.iter().map(|l| l.as_ptr()).collect_vec();

    Ok((requested_layers, requested_extensions))
  }

  fn supported(entry: &ash::Entry) -> Result<(Vec<vk::LayerProperties>, Vec<vk::ExtensionProperties>), VulkanError> {
    let layers = entry.enumerate_instance_layer_properties()?;
    let extensions = entry.enumerate_instance_extension_properties(None)?;

    Ok((layers, extensions))
  }
}
