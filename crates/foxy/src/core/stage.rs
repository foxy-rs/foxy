use foxy_window::prelude::*;
use strum::{Display, EnumDiscriminants};

use super::framework::FoxyFramework;

// KEEP THESE SMALL since you need to clone them for each iteration
#[derive(Display, EnumDiscriminants)]
pub enum Stage<'s> {
  Initializing,
  Start {
    foxy: &'s mut FoxyFramework,
  },
  BeginFrame {
    foxy: &'s mut FoxyFramework,
    message: &'s mut WindowMessage,
  },
  EarlyUpdate {
    foxy: &'s mut FoxyFramework,
    message: &'s mut WindowMessage,
  },
  FixedUpdate {
    foxy: &'s mut FoxyFramework,
  },
  Update {
    foxy: &'s mut FoxyFramework,
    message: &'s mut WindowMessage,
  },
  EndFrame {
    foxy: &'s mut FoxyFramework,
    message: &'s mut WindowMessage,
  },
  Exiting {
    foxy: &'s mut FoxyFramework,
  },
  ExitLoop,
}
