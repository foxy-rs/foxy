use ash::{prelude::*, vk};
use itertools::Itertools;
// use ezwin::window::Window;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle};
use std::ffi::{c_char, CStr};
use tracing::*;

use crate::{
  builder::{MissingWindow, ValidationStatus, VulkanBuilder, VulkanCreateInfo},
  error::VulkanError,
};

pub struct Vulkan {
  instance: ash::Instance,
}

impl Drop for Vulkan {
  fn drop(&mut self) {
    trace!("Dropping Vulkan");
    unsafe {
      self.instance.destroy_instance(None);
    }
  }
}

impl Vulkan {
  #[cfg(not(debug_assertions))]
  const VALIDATION_LAYERS: ValidationLayers = ValidationLayers(&[]);
  #[cfg(debug_assertions)]
  const VALIDATION_LAYERS: &'static [&'static CStr] = &[c"VK_LAYER_KHRONOS_validation"];

  const EXTENSIONS: &'static [&'static CStr] = &[c"VK_KHR_surface"];

  pub fn builder() -> VulkanBuilder<MissingWindow> {
    Default::default()
  }

  pub(crate) fn new<W: HasRawDisplayHandle + HasRawWindowHandle>(
    create_info: VulkanCreateInfo<W>,
  ) -> Result<Self, VulkanError> {
    let display_handle = create_info.window.raw_display_handle();
    trace!("Initializing Vulkan");
    let instance = Self::create_instance(display_handle, create_info.validation_status)?;
    Ok(Self { instance })
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

  fn create_instance(
    display_handle: RawDisplayHandle,
    validation_status: ValidationStatus,
  ) -> Result<ash::Instance, VulkanError> {
    let entry = ash::Entry::linked();
    let version = Self::select_version(&entry);

    let app_info = vk::ApplicationInfo::default()
      .api_version(version)
      .engine_name(c"Ookami")
      .engine_version(vk::make_api_version(0, 1, 0, 0))
      .application_name(c"Ookami App")
      .application_version(vk::make_api_version(0, 1, 0, 0));

    // let layers: Vec<*const c_char> = Self::VALIDATION_LAYERS.iter().map(|name| name.as_ptr()).collect();
    let extensions: Vec<*const c_char> = ash_window::enumerate_required_extensions(display_handle)?.to_vec();

    let (requested_layers, requested_extensions) = Self::requested(&entry, validation_status)?;

    let instance_create_info = vk::InstanceCreateInfo::default()
      .application_info(&app_info)
      .enabled_layer_names(&requested_layers)
      .enabled_extension_names(&extensions);

    let instance = unsafe { entry.create_instance(&instance_create_info, None)? };

    Ok(instance)
  }

  fn requested(
    entry: &ash::Entry,
    validation_status: ValidationStatus,
  ) -> Result<(Vec<*const c_char>, Vec<*const c_char>), VulkanError> {
    let (supported_layers, supported_extensions) = Self::supported(entry)?;

    let mut layers = supported_layers
      .iter()
      .cartesian_product(Self::VALIDATION_LAYERS)
      .filter_map(|(supported_layer, &layer)| {
        if supported_layer.layer_name_as_c_str().unwrap() == layer {
          Some(layer.as_ptr())
        } else {
          None
        }
      })
      .collect_vec();

    if layers.len() != Self::VALIDATION_LAYERS.len() {
      return Err(VulkanError::Unsupported(
        "not all requested layers are supported on this device",
      ));
    }

    if validation_status == ValidationStatus::Disabled {
      layers.clear();
    }

    let extensions = supported_extensions
      .iter()
      .cartesian_product(Self::EXTENSIONS)
      .filter_map(|(supported_extension, &extension)| {
        if supported_extension.extension_name_as_c_str().unwrap() == extension {
          Some(extension.as_ptr())
        } else {
          None
        }
      })
      .collect_vec();

    if extensions.len() != Self::EXTENSIONS.len() {
      return Err(VulkanError::Unsupported(
        "not all requested extensions are supported on this device",
      ));
    }

    Ok((layers, extensions))
  }

  fn supported(entry: &ash::Entry) -> Result<(Vec<vk::LayerProperties>, Vec<vk::ExtensionProperties>), VulkanError> {
    // let layers: Vec<*const c_char> = Self::VALIDATION_LAYERS.iter().map(|name| name.as_ptr()).collect();
    let layers = unsafe { entry.enumerate_instance_layer_properties() }?;
    let extensions = unsafe { entry.enumerate_instance_extension_properties(None) }?;

    Ok((layers, extensions))
  }
}
