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
