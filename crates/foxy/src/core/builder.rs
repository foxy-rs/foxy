use foxy_utils::time::TimeCreateInfo;
use winit::dpi::{LogicalSize, Size};

use crate::window::WindowCreateInfo;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum Polling {
  Poll,
  #[default]
  Wait,
}

#[derive(Debug, Default)]
pub enum DebugInfo {
  Shown,
  #[default]
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
  pub fn with_window_info(mut self, window: WindowCreateInfo) -> Self {
    self.window = window;
    self
  }

  pub fn with_title(mut self, title: String) -> Self {
    self.window.title = title;
    self
  }

  pub fn with_size(mut self, width: u32, height: u32) -> Self {
    self.window.inner_size = Some(Size::Logical(LogicalSize {
        width,
        height,
      }));
    self
  }

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
