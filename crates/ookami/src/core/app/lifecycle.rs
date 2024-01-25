use foxy_window::prelude::*;

#[derive(Debug)]
pub enum Lifecycle {
  Entering,
  BeginFrame { message: WindowMessage },
  EarlyUpdate { message: WindowMessage },
  FixedUpdate { message: WindowMessage },
  Update { message: WindowMessage },
  EndFrame { message: WindowMessage },
  Exiting,
  ExitLoop,
}
