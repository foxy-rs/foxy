use foxy_utils::time::TimeCreateInfo;

use crate::window::create_info::WindowCreateInfo;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum Polling {
  #[default]
  Poll,
  Wait,
}

#[derive(Debug, Default)]
pub enum DebugInfo {
  #[default]
  Shown,
  Hidden,
}

#[derive(Debug, Default)]
pub struct FoxyCreateInfo {
  pub time: TimeCreateInfo,
  pub window: WindowCreateInfo,
  pub polling_strategy: Polling,
  pub debug_info: DebugInfo,
}

impl FoxyCreateInfo {
  pub fn with_polling(mut self, polling_strategy: Polling) -> Self {
    self.polling_strategy = polling_strategy;
    self
  }

  pub fn with_debug_info(mut self, debug_info: DebugInfo) -> Self {
    self.debug_info = debug_info;
    self
  }

  pub fn with_time(mut self, time: TimeCreateInfo) -> Self {
    self.time = time;
    self
  }
}
