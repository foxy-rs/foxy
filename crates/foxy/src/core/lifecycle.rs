use foxy_window::prelude::*;

// KEEP THESE SMALL since you need to clone them for each iteration
#[derive(Debug, Clone)]
pub enum Stage {
  Initializing,
  Start,
  BeginFrame { message: WindowMessage },
  EarlyUpdate { message: WindowMessage },
  FixedUpdate { message: WindowMessage },
  Update { message: WindowMessage },
  EndFrame { message: WindowMessage },
  Exiting,
  ExitLoop,
}
