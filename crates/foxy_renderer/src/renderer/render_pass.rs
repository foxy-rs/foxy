use uuid::Uuid;
use wgpu::CommandEncoder;

use super::target::RenderTarget;
use crate::{
  error::RendererError,
  renderer::{asset_manager::AssetManager, mesh::BakedStaticMesh},
};

pub mod simple;
pub mod tonemap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PassHandle(pub Uuid);

impl From<Uuid> for PassHandle {
  fn from(value: Uuid) -> Self {
    Self(value)
  }
}

pub trait Pass {
  fn draw(
    &mut self,
    command_encoder: &mut CommandEncoder,
    asset_manager: &AssetManager,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view: &wgpu::TextureView,
    mesh: &BakedStaticMesh,
  ) -> Result<(), RendererError>;

  fn resize(&mut self, device: &wgpu::Device, render_target: &RenderTarget);
}

// pub fn create_render_pipeline(
//   label: Option<&str>,
//   device: &wgpu::Device,
//   layout: &wgpu::PipelineLayout,
//   color_format: wgpu::TextureFormat,
//   depth_format: Option<wgpu::TextureFormat>,
//   vertex_layouts: &[wgpu::VertexBufferLayout],
//   shader: wgpu::ShaderModule,
// ) -> wgpu::RenderPipeline {
//   // let shader = device.create_shader_module(shader);
//
//   device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
//     label,
//     layout: Some(layout),
//     vertex: wgpu::VertexState {
//       module: &shader,
//       entry_point: "vs_main",
//       buffers: vertex_layouts,
//     },
//     fragment: Some(wgpu::FragmentState {
//       module: &shader,
//       entry_point: "fs_main",
//       targets: &[Some(wgpu::ColorTargetState {
//         format: color_format,
//         blend: Some(wgpu::BlendState {
//           alpha: wgpu::BlendComponent::REPLACE,
//           color: wgpu::BlendComponent::REPLACE,
//         }),
//         write_mask: wgpu::ColorWrites::ALL,
//       })],
//     }),
//     primitive: wgpu::PrimitiveState {
//       topology: PrimitiveTopology::TriangleList,
//       strip_index_format: None,
//       front_face: wgpu::FrontFace::Ccw,
//       cull_mode: Some(wgpu::Face::Back),
//       // Setting this to anything other than Fill requires
// Features::NON_FILL_POLYGON_MODE       polygon_mode: wgpu::PolygonMode::Fill,
//       // Requires Features::DEPTH_CLIP_CONTROL
//       unclipped_depth: false,
//       // Requires Features::CONSERVATIVE_RASTERIZATION
//       conservative: false,
//     },
//     depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
//       format,
//       depth_write_enabled: true,
//       depth_compare: wgpu::CompareFunction::Less,
//       stencil: wgpu::StencilState::default(),
//       bias: wgpu::DepthBiasState::default(),
//     }),
//     multisample: wgpu::MultisampleState {
//       count: 1,
//       mask: !0,
//       alpha_to_coverage_enabled: false,
//     },
//     // If the pipeline will be used with a multiview render pass, this
//     // indicates how many array layers the attachments will have.
//     multiview: None,
//   })
// }
