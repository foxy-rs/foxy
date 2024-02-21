use std::sync::Arc;

use wgpu::TextureFormat;

use super::{shader::ShaderHandle, texture::TextureHandle, Renderer};

// #[repr(C)]
// pub struct MaterialUniforms {
//   pub color: [f32; 4],
// }

pub trait Material {
  // fn id() -> Uuid;

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
      albedo: TextureHandle(albedo.unwrap_or("assets/foxy/textures/default.png").into()),
    })
  }
}
