use ash::vk;
use foxy_utils::types::handle::Handle;

use super::device::Device;

#[derive(Debug, Default)]
pub struct FrameData {
  pub command_pool: vk::CommandPool,
  pub master_command_buffer: vk::CommandBuffer,
}

impl FrameData {
  pub const FRAME_OVERLAP: usize = 2;

  pub fn delete(&mut self, device: &mut Handle<Device>) {
    unsafe {
      device.get_mut().logical().destroy_command_pool(self.command_pool, None);
    }
  }
}
