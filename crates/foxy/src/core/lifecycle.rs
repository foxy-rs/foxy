use foxy_window::prelude::*;

#[derive(Debug)]
pub enum Lifecycle {
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
