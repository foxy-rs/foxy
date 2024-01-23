use ash::{prelude::*, vk};
use ezwin::window::Window;
use raw_window_handle::HasRawDisplayHandle;
use std::ffi::{c_char, CStr};
use tracing::*;

pub struct Vulkan {
  instance: ash::Instance,
}

impl Drop for Vulkan {
  fn drop(&mut self) {
    unsafe {
      self.instance.destroy_instance(None);
    }
  }
}

impl Vulkan {
  #[cfg(not(debug_assertions))]
  const VALIDATION_LAYERS: &'static [&'static CStr] = &[];
  #[cfg(debug_assertions)]
  const VALIDATION_LAYERS: &'static [&'static CStr] = &[c"VK_LAYER_KHRONOS_validation"];

  pub fn new(window: &Window) -> anyhow::Result<Self> {
    let instance = Self::create_instance(window)?;
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

    let version = vk::make_api_version(0, major, minor, 0);

    {
      let variant = vk::api_version_variant(version);
      let major = vk::api_version_major(version);
      let minor = vk::api_version_minor(version);
      let patch = vk::api_version_patch(version);

      info!("Selected version: Vulkan {major}.{minor}.{patch}.{variant}");
    }

    version
  }

  fn create_instance(window: &Window) -> anyhow::Result<ash::Instance> {
    trace!("Creating Vulkan instance");

    let entry = ash::Entry::linked();
    let version = Self::select_version(&entry);

    let app_info = vk::ApplicationInfo::default()
      .api_version(version)
      .engine_name(c"Ookami")
      .engine_version(vk::make_api_version(0, 1, 0, 0))
      .application_name(c"Ookami App")
      .application_version(vk::make_api_version(0, 1, 0, 0));

    let layers: Vec<*const c_char> = Self::VALIDATION_LAYERS.iter().map(|name| name.as_ptr()).collect();
    let extensions = ash_window::enumerate_required_extensions(window.raw_display_handle())
      .unwrap()
      .to_vec();
    
    let instance_create_info = vk::InstanceCreateInfo::default()
      .application_info(&app_info)
      .enabled_layer_names(&layers)
      .enabled_extension_names(&extensions);

    let instance = unsafe { entry.create_instance(&instance_create_info, None)? };

    Ok(instance)
  }
}
