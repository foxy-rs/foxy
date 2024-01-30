pub use foxy_utils::types::behavior::{CloseBehavior, ColorMode, Visibility};

pub use crate::{
  debug::validation::ValidationLayer,
  window::{
    builder::WindowBuilder,
    message::{AppMessage, KeyboardMessage, MouseMessage, WindowMessage},
    Window,
  },
};
