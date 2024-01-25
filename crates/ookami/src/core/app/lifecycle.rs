use foxy_window::prelude::*;

#[derive(Debug)]
pub enum Lifecycle {
  Entering,
  BeginFrame { message: Option<WindowMessage> },
  EarlyUpdate { message: Option<WindowMessage> },
  FixedUpdate { message: Option<WindowMessage> },
  Update { message: Option<WindowMessage> },
  EndFrame { message: Option<WindowMessage> },
  Exiting,
  ExitLoop,
}
