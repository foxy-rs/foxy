use foxy_time::TimeSettings;
use winit::dpi::{PhysicalSize, Size};

pub struct WindowSettings {
  pub title: String,
  pub size: Size,
  pub is_visible: bool,
  pub should_wait: bool,
  pub should_close_on_x: bool,
}

impl Default for WindowSettings {
  fn default() -> Self {
    Self {
      title: "Foxy Window".to_owned(),
      size: PhysicalSize::new(800, 600).into(),
      is_visible: true,
      should_wait: true,
      should_close_on_x: true,
    }
  }
}

impl WindowSettings {
  pub fn with_title(mut self, title: impl Into<String>) -> Self {
    self.title = title.into();
    self
  }

  pub fn with_size(mut self, size: impl Into<Size>) -> Self {
    self.size = size.into();
    self
  }

  pub fn with_visible(mut self, is_visible: bool) -> Self {
    self.is_visible = is_visible;
    self
  }

  pub fn with_should_wait(mut self, should_wait: bool) -> Self {
    self.should_wait = should_wait;
    self
  }

  pub fn with_close_on_x(mut self, should_close_on_x: bool) -> Self {
    self.should_close_on_x = should_close_on_x;
    self
  }
}

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

#[derive(Default)]
pub struct FoxySettings {
  pub time: TimeSettings,
  pub window: WindowSettings,
  pub debug_info: DebugInfo,
}

impl FoxySettings {
  pub fn with_window(mut self, window: WindowSettings) -> Self {
    self.window = window;
    self
  }

  pub fn with_time(mut self, time: TimeSettings) -> Self {
    self.time = time;
    self
  }

  pub fn with_debug_info(mut self, debug_info: DebugInfo) -> Self {
    self.debug_info = debug_info;
    self
  }
}
