use std::ffi::{c_char, CStr};
use raw_window_handle::HasRawDisplayHandle;
use ash::{prelude::*, vk};
use ezwin::window::Window;
use tracing::*;

pub struct Vulkan {
  instance: vk::Instance,
}

impl Vulkan {
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

  fn create_instance(window: &Window) -> anyhow::Result<vk::Instance> {
    trace!("Creating Vulkan instance");
    
    let entry = ash::Entry::linked();
    let version = Self::select_version(&entry);

    let app_info = vk::ApplicationInfo::default()
      .api_version(version)
      .engine_name(unsafe { CStr::from_bytes_with_nul_unchecked(b"Ookami\0") })
      .engine_version(vk::make_api_version(0, 1, 0, 0))
      .application_name(unsafe { CStr::from_bytes_with_nul_unchecked(b"Ookami App\0") })
      .application_version(vk::make_api_version(0, 1, 0, 0));

    if cfg!(debug_assertions) {
      let layers = [unsafe { CStr::from_bytes_with_nul_unchecked(b"VK_LAYER_KHRONOS_validation\0") }];
      let layers: Vec<*const c_char> = layers.iter().map(|name| name.as_ptr()).collect();
      let extensions = ash_window::enumerate_required_extensions(window.raw_display_handle())
        .unwrap()
        .to_vec();
      info!("Extensions: {extensions:?}");
    }

    Ok(vk::Instance::null())
  }
}
