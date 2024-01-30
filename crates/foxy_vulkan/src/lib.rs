#![deny(unsafe_op_in_unsafe_fn)]
#![feature(associated_type_defaults)]
#![warn(clippy::indexing_slicing)]

pub mod buffer;
pub mod command_buffer;
pub mod deletion_queue;
pub mod device;
pub mod error;
pub mod image;
pub mod image_format;
pub mod pipeline;
pub mod shader;
pub mod surface;
pub mod swapchain;
pub mod sync_objects;
