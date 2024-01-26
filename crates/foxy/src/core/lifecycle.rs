use foxy_window::prelude::*;
use strum::Display;

use super::FoxyFramework;

// KEEP THESE SMALL since you need to clone them for each iteration
#[derive(Display)]
pub enum Stage {
  Initializing,
  Start {
    foxy: FoxyFramework,
  },
  BeginFrame {
    foxy: FoxyFramework,
    message: WindowMessage,
  },
  EarlyUpdate {
    foxy: FoxyFramework,
    message: WindowMessage,
  },
  FixedUpdate {
    foxy: FoxyFramework,
    message: WindowMessage,
  },
  Update {
    foxy: FoxyFramework,
    message: WindowMessage,
  },
  EndFrame {
    foxy: FoxyFramework,
    message: WindowMessage,
  },
  Exiting {
    foxy: FoxyFramework,
  },
  ExitLoop,
}
