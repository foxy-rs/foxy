use thiserror::Error;

use crate::vulkan::error::VulkanError;

#[derive(Error, Debug)]
pub enum RendererError {
  #[error("{0}")]
  Recoverable(Recoverable),
  #[error("{0}")]
  Unrecoverable(Unrecoverable),
}

#[derive(Error, Debug)]
pub enum Recoverable {
  #[error("{0}")]
  Vulkan(#[from] VulkanError),
  #[error("{0}")]
  Ash(#[from] ash::vk::Result),
}

#[derive(Error, Debug)]
pub enum Unrecoverable {
  #[error("{0}")]
  Vulkan(#[from] VulkanError),
  #[error("{0}")]
  Ash(#[from] ash::vk::Result),
}