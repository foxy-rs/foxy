use std::sync::Arc;

use wgpu::TextureFormat;

use super::{diffuse_texture::TextureHandle, shader::ShaderHandle, Renderer};

pub trait Material {
  fn format() -> TextureFormat
  where
    Self: Sized,
  {
    Renderer::SURFACE_FORMAT
  }

  fn albedo(&self) -> TextureHandle;
  fn shader(&self) -> ShaderHandle;
}

pub struct StandardMaterial {
  albedo: TextureHandle,
}

impl Material for StandardMaterial {
  fn albedo(&self) -> TextureHandle {
    self.albedo.clone()
  }

  fn shader(&self) -> ShaderHandle {
    "assets/foxy/shaders/texture.wgsl".into()
  }
}

impl StandardMaterial {
  pub fn new(albedo: Option<&'static str>) -> Arc<Self> {
    Arc::new(Self {
      albedo: TextureHandle::FromFile(albedo.unwrap_or("assets/foxy/textures/default.png").into()),
    })
  }
}
