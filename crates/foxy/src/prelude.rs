pub use foxy_utils::log::prelude::*;

pub use crate::core::{
  builder::{DebugInfo, FoxyCreateInfo, Polling},
  foxy_state::Foxy,
  event::{FoxyEvent, InputEvent, WindowEvent},
  foxy_loop::Framework,
  message::RenderLoopMessage,
  runnable::Runnable,
  FoxyResult,
};
