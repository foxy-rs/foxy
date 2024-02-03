#![deny(unsafe_op_in_unsafe_fn)]
#![feature(associated_type_defaults)]
#![warn(clippy::indexing_slicing)]

pub mod error;
pub mod renderer;
pub mod vulkan;

#[macro_export]
macro_rules! include_shader {
  ($type:tt; $device:expr, $path:expr) => {{
    let pathbuf = std::path::PathBuf::from($path);
    let srcstr = include_str!($path);
    let source = $crate::vulkan::shader::source::Source::from_source::<$type>(pathbuf.clone(), srcstr);
    $crate::vulkan::shader::Shader::from_source::<$type>($device, pathbuf, source)
  }};
}

#[macro_export]
macro_rules! store_shader {
  (<$type:tt>($shader_store:expr, $path:expr)) => {{
    let shader = $crate::include_shader!($type; $shader_store.device(), $path);
    $shader_store.insert::<$type>(shader);
  }};
}
