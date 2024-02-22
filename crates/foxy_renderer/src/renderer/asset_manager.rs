use std::{
  borrow::Cow,
  collections::HashMap,
  fs,
  sync::{Arc, RwLock},
};

use tracing::error;
use uuid::Uuid;
use wgpu::{PrimitiveTopology, RenderPipeline, ShaderModule, ShaderModuleDescriptor, ShaderSource};

use super::shader::ShaderHandle;
use crate::renderer::texture::{DiffuseTexture, TextureHandle};

pub struct RenderPipelineInfo<'a> {
  pub id: Uuid,
  pub label: Option<&'a str>,
  pub layout: &'a wgpu::PipelineLayout,
  pub color_format: wgpu::TextureFormat,
  pub depth_format: Option<wgpu::TextureFormat>,
  pub vertex_layouts: &'a [wgpu::VertexBufferLayout<'a>],
  pub shader: &'a ShaderModule,
}

#[derive(Clone, Default)]
pub struct AssetManager {
  shaders: Arc<RwLock<HashMap<ShaderHandle, Arc<ShaderModule>>>>,
  textures: Arc<RwLock<HashMap<TextureHandle, Arc<DiffuseTexture>>>>,
  render_pipelines: Arc<RwLock<HashMap<Uuid, Arc<RenderPipeline>>>>,
}

impl AssetManager {
  pub fn new() -> Self {
    Self {
      shaders: Arc::new(RwLock::new(Default::default())),
      textures: Arc::new(RwLock::new(Default::default())),
      render_pipelines: Arc::new(RwLock::new(Default::default())),
    }
  }

  pub fn read_shader(&self, shader: ShaderHandle, device: &wgpu::Device) -> Arc<ShaderModule> {
    let contains_key = {
      let shaders = self.shaders.read().unwrap();
      shaders.contains_key(&shader)
    };

    if contains_key {
      // debug!("Reading shader from cache");
      let shaders = self.shaders.read().unwrap();
      shaders.get(&shader).unwrap().clone()
    } else {
      let mut shaders = self.shaders.write().unwrap();

      let path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join(shader.0.clone());

      // debug!("Reading shader from source: {path:?}");

      let source = fs::read_to_string(path).unwrap_or_default();

      let desc = ShaderModuleDescriptor {
        label: Some(shader.0.to_str().unwrap()),
        source: ShaderSource::Wgsl(Cow::Owned(source)),
      };

      let module = device.create_shader_module(desc);

      shaders.insert(shader.clone(), Arc::from(module));

      shaders.get(&shader).unwrap().clone()
    }
  }

  pub fn read_texture(
    &self,
    texture: TextureHandle,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
  ) -> Arc<DiffuseTexture> {
    let contains_key = {
      let textures = self.textures.read().unwrap();
      textures.contains_key(&texture)
    };

    if contains_key {
      let textures = self.textures.read().unwrap();
      textures.get(&texture).unwrap().clone()
    } else {
      let mut textures = self.textures.write().unwrap();

      let path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join(texture.0.clone());

      let source = fs::read(path).unwrap_or_else(|error| {
        error!("{error:?}");
        include_bytes!("../../assets/foxy/textures/default.png").to_vec()
      });

      let diffuse = DiffuseTexture::new(device, queue, &source);

      textures.insert(texture.clone(), Arc::from(diffuse));

      textures.get(&texture).unwrap().clone()
    }
  }

  pub fn create_render_pipeline(&self, device: &wgpu::Device, info: &RenderPipelineInfo) -> Arc<RenderPipeline> {
    let contains_key = {
      let pipelines = self.render_pipelines.read().unwrap();
      pipelines.contains_key(&info.id)
    };

    if contains_key {
      let pipelines = self.render_pipelines.read().unwrap();
      pipelines.get(&info.id).unwrap().clone()
    } else {
      let mut pipelines = self.render_pipelines.write().unwrap();

      let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: info.label,
        layout: Some(info.layout),
        vertex: wgpu::VertexState {
          module: info.shader,
          entry_point: "vs_main",
          buffers: info.vertex_layouts,
        },
        fragment: Some(wgpu::FragmentState {
          module: info.shader,
          entry_point: "fs_main",
          targets: &[Some(wgpu::ColorTargetState {
            format: info.color_format,
            blend: Some(wgpu::BlendState {
              alpha: wgpu::BlendComponent::REPLACE,
              color: wgpu::BlendComponent::REPLACE,
            }),
            write_mask: wgpu::ColorWrites::ALL,
          })],
        }),
        primitive: wgpu::PrimitiveState {
          topology: PrimitiveTopology::TriangleList,
          strip_index_format: None,
          front_face: wgpu::FrontFace::Ccw,
          cull_mode: Some(wgpu::Face::Back),
          // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
          polygon_mode: wgpu::PolygonMode::Fill,
          // Requires Features::DEPTH_CLIP_CONTROL
          unclipped_depth: false,
          // Requires Features::CONSERVATIVE_RASTERIZATION
          conservative: false,
        },
        depth_stencil: info.depth_format.map(|format| wgpu::DepthStencilState {
          format,
          depth_write_enabled: true,
          depth_compare: wgpu::CompareFunction::Less,
          stencil: wgpu::StencilState::default(),
          bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
          count: 1,
          mask: !0,
          alpha_to_coverage_enabled: false,
        },
        // If the pipeline will be used with a multiview render pass, this
        // indicates how many array layers the attachments will have.
        multiview: None,
      });

      pipelines.insert(info.id, Arc::from(pipeline));

      pipelines.get(&info.id).unwrap().clone()
    }
  }
}

// pub enum CreationResult<T> {
//   New(Uuid, T),
//   Cached(T),
// }

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// pub struct PipelineHandle {
//   path: PathBuf,
// }
//
// impl From<&str> for crate::renderer::shader::ShaderHandle {
//   fn from(value: &str) -> Self {
//     Self { path: value.into() }
//   }
// }
