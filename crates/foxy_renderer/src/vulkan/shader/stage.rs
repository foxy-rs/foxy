use std::ffi::CString;

use ash::vk;
use strum::{Display, EnumIter};

pub mod compute;
pub mod fragment;
pub mod geometry;
pub mod mesh;
pub mod vertex;

#[derive(EnumIter, Display, Clone, Debug, PartialEq, Eq, Hash)]
#[strum(serialize_all = "snake_case")]
pub enum ShaderKind {
  Vertex,
  Fragment,
  Compute,
  Geometry,
  Mesh,
}

impl ShaderKind {
  pub fn entry_point(&self) -> String {
    // self.to_string() + "_main"
    "main".into()
  }

  pub fn entry_point_cstring(&self) -> CString {
    CString::new(self.entry_point()).unwrap()
  }
}

impl From<ShaderKind> for shaderc::ShaderKind {
  fn from(value: ShaderKind) -> Self {
    match value {
      ShaderKind::Vertex => shaderc::ShaderKind::Vertex,
      ShaderKind::Fragment => shaderc::ShaderKind::Fragment,
      ShaderKind::Compute => shaderc::ShaderKind::Compute,
      ShaderKind::Geometry => shaderc::ShaderKind::Geometry,
      ShaderKind::Mesh => shaderc::ShaderKind::Mesh,
    }
  }
}

impl From<ShaderKind> for vk::ShaderStageFlags {
  fn from(value: ShaderKind) -> Self {
    match value {
      ShaderKind::Vertex => vk::ShaderStageFlags::VERTEX,
      ShaderKind::Fragment => vk::ShaderStageFlags::FRAGMENT,
      ShaderKind::Compute => vk::ShaderStageFlags::COMPUTE,
      ShaderKind::Geometry => vk::ShaderStageFlags::GEOMETRY,
      ShaderKind::Mesh => vk::ShaderStageFlags::MESH_NV,
    }
  }
}

pub trait StageInfo {
  fn kind() -> ShaderKind;
  fn default_source() -> String;
}
