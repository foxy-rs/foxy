use thiserror::Error;

use crate::vulkan::error::VulkanError;



#[derive(Error, Debug)]
pub enum RendererError {
  #[error("{0}")]
  Vulkan(#[from] VulkanError),
}