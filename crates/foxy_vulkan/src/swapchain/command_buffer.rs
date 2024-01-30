use ash::vk;
use foxy_utils::types::handle::Handle;

use crate::{device::Device, error::VulkanError, swapchain::Swapchain, vulkan_error};

use super::pipeline::RenderPipeline;

#[derive(Clone)]
pub struct CommandBuffers {
  device: Handle<Device>,
  swapchain: Handle<Swapchain>,
  buffers: Vec<vk::CommandBuffer>,
}

impl CommandBuffers {
  pub fn new(device: Handle<Device>, swapchain: Handle<Swapchain>) -> Result<Self, VulkanError> {
    let buffer_info = vk::CommandBufferAllocateInfo::default()
      .level(vk::CommandBufferLevel::PRIMARY)
      .command_pool(*device.get().command_pool())
      .command_buffer_count(swapchain.get().image_count() as u32);
    let buffers = unsafe { device.get().logical().allocate_command_buffers(&buffer_info) }?;

    Ok(Self {
      device,
      swapchain,
      buffers,
    })
  }

  pub fn delete(&mut self) {}

  pub fn record(&self, pipeline: &impl RenderPipeline) -> Result<(), VulkanError> {
    // record a hard-coded set of commands for now

    let clear_values: [vk::ClearValue; 2] = [
      vk::ClearValue {
        color: vk::ClearColorValue {
          float32: [0.1, 0.1, 0.1, 0.1],
        },
      },
      vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue { depth: 1.0, stencil: 0 },
      },
    ];

    for (i, &buffer) in self.buffers.iter().enumerate() {
      let begin_info = vk::CommandBufferBeginInfo::default();

      unsafe { self.device.get().logical().begin_command_buffer(buffer, &begin_info) }?;

      let framebuffer = self
        .swapchain
        .get()
        .frame_buffer(i)
        .ok_or_else(|| vulkan_error!("invalid framebuffer index"))?;

      let pass_info = vk::RenderPassBeginInfo::default()
        .render_pass(self.swapchain.get().render_pass())
        .framebuffer(framebuffer)
        .render_area(
          vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(vk::Extent2D {
              width: self.swapchain.get().size().width as u32,
              height: self.swapchain.get().size().height as u32,
            }),
        )
        .clear_values(&clear_values);

      unsafe {
        self
          .device
          .get()
          .logical()
          .cmd_begin_render_pass(buffer, &pass_info, vk::SubpassContents::INLINE)
      };

      pipeline.bind(buffer);

      unsafe { self.device.get().logical().cmd_draw(buffer, 3, 1, 0, 0) };

      unsafe { self.device.get().logical().cmd_end_render_pass(buffer) };

      unsafe { self.device.get().logical().end_command_buffer(buffer) }?;
    }

    Ok(())
  }

  pub fn submit(&mut self, image_index: usize) -> Result<bool, VulkanError> {
    self
      .swapchain
      .get_mut()
      .submit_command_buffers(&self.buffers, image_index)
  }

  pub fn buffers(&self) -> &[vk::CommandBuffer] {
    &self.buffers
  }

  pub fn buffer(&self, index: usize) -> Option<&vk::CommandBuffer> {
    self.buffers.get(index)
  }
}
