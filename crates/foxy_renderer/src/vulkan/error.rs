use std::ffi::CStr;

use ash::{extensions::ext, vk};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VulkanError {
  #[error("VkResult: `{0}`")]
  Vulkan(#[from] ash::vk::Result),
  #[error("{0}")]
  Shaderc(#[from] shaderc::Error),
  #[error("{0}")]
  Unsupported(String),
  #[error("{0}")]
  Shader(String),
  #[error("{0}")]
  Other(#[from] anyhow::Error),
  #[error("{0}")]
  SyncObjects(String),
  #[error("{0}")]
  IO(#[from] std::io::Error),
}

#[macro_export]
macro_rules! vulkan_unsupported_error {
  () => {
    $crate::vulkan::error::VulkanError::Unsupported(format!("attempted action unsupported by the device running Vulkan"))
  };
  ($($arg:tt)*) => {{
    $crate::vulkan::error::VulkanError::Unsupported(format!($($arg)*))
  }}
}

#[macro_export]
macro_rules! vulkan_shader_error {
  () => {
    $crate::vulkan::error::VulkanError::Unsupported(format!("attempted action unsupported by the device running Vulkan"))
  };
  ($($arg:tt)*) => {{
    $crate::vulkan::error::VulkanError::Unsupported(format!($($arg)*))
  }}
}

#[macro_export]
macro_rules! vulkan_error {
  () => {
    $crate::vulkan::error::VulkanError::Other(anyhow::anyhow!("vulkan error"))
  };
  ($($arg:tt)*) => {{
    $crate::vulkan::error::VulkanError::Other(anyhow::anyhow!(format!($($arg)*)))
  }}
}

#[derive(Clone)]
pub struct Debug {
  debug_utils: Option<ext::DebugUtils>,
  debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl Debug {
  pub fn delete(&mut self) {
    if let Some(debug_utils) = self.debug_utils.take() {
      if let Some(debug_messenger) = self.debug_messenger.take() {
        unsafe {
          debug_utils.destroy_debug_utils_messenger(debug_messenger, None);
        }
      }
    }
  }
}

impl Debug {
  pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Result<Self, VulkanError> {
    if cfg!(debug_assertions) {
      let debug_utils = ext::DebugUtils::new(entry, instance);

      let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING)
        .message_type(
          vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        )
        .pfn_user_callback(Some(debug_callback));

      let debug_messenger = unsafe { debug_utils.create_debug_utils_messenger(&create_info, None) }?;

      Ok(Self {
        debug_utils: Some(debug_utils),
        debug_messenger: Some(debug_messenger),
      })
    } else {
      Ok(Self {
        debug_utils: None,
        debug_messenger: None,
      })
    }
  }

  // pub fn delete(&mut self) {
  //   if let Some(debug_utils) = self.debug_utils.take() {
  //     if let Some(debug_messenger) = self.debug_messenger.take() {
  //       unsafe { debug_utils.destroy_debug_utils_messenger(debug_messenger,
  // None); }     }
  //   }
  // }
}

unsafe extern "system" fn debug_callback(
  message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
  message_types: vk::DebugUtilsMessageTypeFlagsEXT,
  p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
  _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
  let callback_data = unsafe { *p_callback_data };
  let message = unsafe { CStr::from_ptr(callback_data.p_message) };
  // let message_id_name = unsafe { callback_data.message_id_name_as_c_str() };
  // if let Some(message) =  {
    match message_types {
      vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
          tracing::error!("VULKAN VALIDATION: {message:?}")
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
          tracing::error!("VULKAN VALIDATION: {message:?}")
        }
        _ => {}
      },
      vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
          tracing::error!("VULKAN DEVICE_ADDRESS_BINDING: {message:?}")
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
          tracing::error!("VULKAN DEVICE_ADDRESS_BINDING: {message:?}")
        }
        _ => {}
      },
      vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
          tracing::error!("VULKAN PERFORMANCE: {message:?}")
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
          tracing::error!("VULKAN PERFORMANCE: {message:?}")
        }
        _ => {}
      },
      _ => {}
    }
  // }

  false.into()
}
