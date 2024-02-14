use std::sync::Arc;

use wgpu::{include_wgsl, Device, Queue, TextureFormat};

use super::{context::GraphicsContext, texture::DiffuseTexture, vertex::Vertex, Renderer};

#[repr(C)]
pub struct MaterialUniforms {
  pub color: [f32; 4],
}

pub trait Material {
  fn format() -> TextureFormat
  where
    Self: Sized,
  {
    GraphicsContext::SURFACE_FORMAT
  }

  fn albedo(&self) -> &DiffuseTexture;
}

pub struct StandardMaterial {
  // pub uniforms: MaterialUniforms,
  // pub uniforms_buffer: wgpu::Buffer,
  pub albedo: DiffuseTexture,
}

impl Material for StandardMaterial {
  fn albedo(&self) -> &DiffuseTexture {
    &self.albedo
  }
}

impl StandardMaterial {
  pub fn new(device: &Device, queue: &Queue, texture: Option<DiffuseTexture>) -> Arc<Self> {
    let albedo = match texture {
      Some(texture) => texture,
      None => DiffuseTexture::new(device, queue, include_bytes!("../../assets/textures/default.png")),
    };

    Arc::new(Self { albedo })
  }
}
