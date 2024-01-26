use thiserror::Error;

#[derive(Error, Debug)]
pub enum VulkanError {
  #[error("VkResult: `{0}`")]
  Vulkan(#[from] ash::vk::Result),
  #[error("{0}")]
  Other(String),
}
