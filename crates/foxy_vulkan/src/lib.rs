#![deny(unsafe_op_in_unsafe_fn)]
#![feature(associated_type_defaults)]
#![warn(clippy::indexing_slicing)]

pub mod deletion_queue;
pub mod device;
pub mod error;
pub mod instance;
pub mod shader;
pub mod surface;
pub mod swapchain;
