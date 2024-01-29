use foxy_window::prelude::*;
use strum::{Display, EnumDiscriminants};

use super::{engine::Foxy, message::RenderLoopMessage};

// KEEP THESE SMALL since you need to clone them for each iteration
#[derive(Display, EnumDiscriminants)]
pub enum Stage<'s> {
  /// Never gets sent to clients
  Initialize,
  Start {
    foxy: &'s mut Foxy,
  },
  BeginFrame {
    foxy: &'s mut Foxy,
    message: &'s mut WindowMessage,
  },
  EarlyUpdate {
    foxy: &'s mut Foxy,
    message: &'s mut WindowMessage,
  },
  FixedUpdate {
    foxy: &'s mut Foxy,
  },
  Update {
    foxy: &'s mut Foxy,
    message: &'s mut WindowMessage,
  },
  EndFrame {
    foxy: &'s mut Foxy,
    message: &'s mut WindowMessage,
    render_response: RenderLoopMessage,
  },
  Exiting {
    foxy: &'s mut Foxy,
  },
  /// Never gets sent to clients
  ExitLoop,
}
