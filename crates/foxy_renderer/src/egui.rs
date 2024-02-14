use std::sync::Arc;

use egui::{epaint::Shadow, Context, FullOutput, RawInput, Rounding, Visuals};
use egui_wgpu::{Renderer, ScreenDescriptor};
use egui_winit::State;
use wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use winit::{event::WindowEvent, window::Window};

pub struct EguiRenderer {
  window: Arc<Window>,
  pub context: Context,
  pub state: State,
  renderer: Renderer,
}

impl EguiRenderer {
  pub fn new(
    window: Arc<Window>,
    device: &Device,
    egui_context: Context,
    output_color_format: TextureFormat,
    output_depth_format: Option<TextureFormat>,
    msaa_samples: u32,
  ) -> EguiRenderer {
    
    let egui_renderer = Renderer::new(device, output_color_format, output_depth_format, msaa_samples);

    let id = egui_context.viewport_id();

    const BORDER_RADIUS: f32 = 6.0;

    let visuals = Visuals {
      window_rounding: Rounding::same(BORDER_RADIUS),
      menu_rounding: Rounding::same(BORDER_RADIUS),
      window_shadow: Shadow::NONE,
      ..Default::default()
    };

    egui_context.set_visuals(visuals);

    let egui_state = egui_winit::State::new(egui_context.clone(), id, &window, None, None);

    EguiRenderer {
      window,
      context: egui_context,
      state: egui_state,
      renderer: egui_renderer,
    }
  }

  pub fn handle_input(&mut self, event: &WindowEvent) -> bool {
    let response = self.state.on_window_event(&self.window, event);

    if response.repaint {
      self.context.request_repaint();
    }

    response.consumed
  }

  pub fn draw(
    &mut self,
    device: &Device,
    queue: &Queue,
    encoder: &mut CommandEncoder,
    window_surface_view: &TextureView,
    screen_descriptor: ScreenDescriptor,
    full_output: FullOutput,
  ) {
    self.state.handle_platform_output(&self.window, full_output.platform_output);

    let tris = self
      .context
      .tessellate(full_output.shapes, full_output.pixels_per_point);

    for (id, image_delta) in &full_output.textures_delta.set {
      self.renderer.update_texture(device, queue, *id, image_delta);
    }

    self
      .renderer
      .update_buffers(device, queue, encoder, &tris, &screen_descriptor);

    {
      let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("EGUI Pass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: window_surface_view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      });
      self.renderer.render(&mut rpass, &tris, &screen_descriptor);
    }

    for x in &full_output.textures_delta.free {
      self.renderer.free_texture(x)
    }
  }
}
