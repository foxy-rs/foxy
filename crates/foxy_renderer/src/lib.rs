#![deny(unsafe_op_in_unsafe_fn)]
#![feature(associated_type_defaults)]
#![warn(clippy::indexing_slicing)]

pub mod error;
pub mod renderer;
pub mod wgpu;