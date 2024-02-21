// #![feature(let_chains)]
// #![feature(duration_constants)]
#![deny(unsafe_op_in_unsafe_fn)]
#![feature(associated_type_defaults)]

pub mod core;
pub mod prelude;
pub mod window;
mod camera;

pub use egui;
pub use foxy_renderer::renderer::{
  material::{Material, StandardMaterial},
  mesh::StaticMesh,
  vertex::Vertex,
};
pub use winit;
