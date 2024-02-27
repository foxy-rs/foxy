use thiserror::Error;

use crate::vulkan::error::VulkanError;

#[derive(Error, Debug)]
pub enum RendererError {
  #[error("{0}")]
  Error(String),
  #[error("must rebuild swapchain")]
  RebuildSwapchain,
  #[error("{0}")]
  Vulkan(#[from] VulkanError),
}

#[macro_export]
macro_rules! renderer_error {
  () => {{
    $crate::error::RendererError::Error("renderer error".to_string())
  }};
  ($($arg:tt)*) => {{
    $crate::error::RendererError::Error(format!($($arg)*))
  }}
}
