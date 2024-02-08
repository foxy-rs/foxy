
use itertools::Itertools;

use crate::vulkan::{device::Device, error::VulkanError};

#[derive(Default)]
pub struct DescriptorLayoutBuilder {
  bindings: Vec<vk::DescriptorSetLayoutBinding>,
}

impl DescriptorLayoutBuilder {
  pub fn new() -> Self {
    Self {
      bindings: Default::default(),
    }
  }

  pub fn add_binding(mut self, binding: u32, kind: vk::DescriptorType) -> Self {
    self.bindings.push(
      *vk::DescriptorSetLayoutBinding::builder()
        .binding(binding)
        .descriptor_count(1)
        .descriptor_type(kind),
    );
    self
  }

  pub fn clear(&mut self) {
    self.bindings.clear();
  }

  pub fn build(
    mut self,
    device: &Device,
    shader_stages: vk::ShaderStageFlags,
  ) -> Result<vk::DescriptorSetLayout, VulkanError> {
    for binding in &mut self.bindings {
      binding.stage_flags |= shader_stages;
    }

    let info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&self.bindings);

    Ok(unsafe { device.logical().create_descriptor_set_layout(&info, None) }?)
  }
}

pub struct PoolSizeRatio {
  pub kind: vk::DescriptorType,
  pub ratio: f32,
}

pub struct DescriptorAllocator {
  device: Device,
  pool: vk::DescriptorPool,
}

impl DescriptorAllocator {
  pub fn new(device: Device, max_sets: u32, pool_ratios: &[PoolSizeRatio]) -> Result<Self, VulkanError> {
    let pool_sizes = pool_ratios
      .iter()
      .map(|ratio| {
        *vk::DescriptorPoolSize::builder()
          .ty(ratio.kind)
          .descriptor_count((ratio.ratio * max_sets as f32) as u32)
      })
      .collect_vec();

    let pool_info = vk::DescriptorPoolCreateInfo::builder()
      .max_sets(max_sets)
      .pool_sizes(&pool_sizes);

    let pool = unsafe { device.logical().create_descriptor_pool(&pool_info, None) }?;

    Ok(Self { device, pool })
  }

  pub fn delete(&mut self) {
    unsafe {
      self.device.logical().destroy_descriptor_pool(self.pool, None);
    }
  }

  pub fn allocate(&self, layout: vk::DescriptorSetLayout) -> Result<vk::DescriptorSet, VulkanError> {
    let layouts = &[layout];
    let alloc_info = *vk::DescriptorSetAllocateInfo::builder()
      .descriptor_pool(self.pool)
      .set_layouts(layouts);

    Ok(
      *unsafe { self.device.logical().allocate_descriptor_sets(&alloc_info) }?
        .first()
        .unwrap(),
    )
  }

  pub fn clear_descriptors(&mut self) -> Result<(), VulkanError> {
    Ok(unsafe {
      self
        .device
        .logical()
        .reset_descriptor_pool(self.pool, vk::DescriptorPoolResetFlags::empty())
    }?)
  }
}
