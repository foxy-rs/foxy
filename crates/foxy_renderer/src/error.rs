use thiserror::Error;

use crate::vulkan::error::VulkanError;

#[derive(Error, Debug)]
pub enum RendererError {
  #[error("{0}")]
  Error(String),
  #[error("{0}")]
  Vulkan(#[from] VulkanError),
}

#[macro_export]
macro_rules! renderer_error {
  () => {
    $crate::error::RendererError::Error("renderer error".to_string())
  };
  ($($arg:tt)*) => {{
    $crate::error::RendererError::Error(format!($($arg)*))
  }}
}

// #[derive(Error, Debug)]
// pub enum Recoverable {
//   #[error("{0}")]
//   Vulkan(#[from] VulkanError),
//   #[error("{0}")]
//   Ash(#[from] ash::vk::Result),
// }

// #[derive(Error, Debug)]
// pub enum Unrecoverable {
//   #[error("{0}")]
//   Vulkan(#[from] VulkanError),
//   #[error("{0}")]
//   Ash(#[from] ash::vk::Result),
// }

// pub trait RendererErr<T> {
//   fn unrecoverable(self) -> Result<T, RendererError>;
//   fn recoverable(self) -> Result<T, RendererError>;
// }

// impl<T> RendererErr<T> for Result<T, VulkanError> {
//   fn unrecoverable(self) -> Result<T, RendererError> {
//     self.map_err(|e| RendererError::Unrecoverable(e.into()))
//   }

//   fn recoverable(self) -> Result<T, RendererError> {
//     self.map_err(|e| RendererError::Recoverable(e.into()))
//   }
// }

// impl<T> RendererErr<T> for Result<T, ash::vk::Result> {
//   fn unrecoverable(self) -> Result<T, RendererError> {
//     self.map_err(|e| RendererError::Unrecoverable(e.into()))
//   }

//   fn recoverable(self) -> Result<T, RendererError> {
//     self.map_err(|e| RendererError::Recoverable(e.into()))
//   }
// }
