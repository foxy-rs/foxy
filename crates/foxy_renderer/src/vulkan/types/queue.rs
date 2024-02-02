use ash::vk;

#[derive(Default, Clone, Copy)]
pub struct Queue {
  queue: vk::Queue,
  family: u32,
}

impl Queue {
  pub fn new(queue: vk::Queue, family: u32) -> Self {
    Self { queue, family }
  }

  pub fn family(&self) -> u32 {
    self.family
  }

  pub fn queue(&self) -> vk::Queue {
    self.queue
  }
}

#[derive(Default)]
pub struct QueueFamilyIndices {
  pub graphics_family: u32,
  pub present_family: u32,
}

impl QueueFamilyIndices {
  // pub fn complete(&self) -> bool { self.graphics_family.is_some() &&
  // self.present_family.is_some() }
}
