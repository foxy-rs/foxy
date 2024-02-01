use foxy_utils::types::behavior::{CloseBehavior, ColorMode, Visibility};

use super::Window;
use crate::debug::error::WindowError;

#[derive(Debug, Clone)]
pub struct HasTitle(pub &'static str);
pub struct MissingTitle;

#[derive(Debug, Clone)]
pub struct HasSize {
  pub width: i32,
  pub height: i32,
}
pub struct MissingSize;

#[derive(Debug, Clone)]
pub struct WindowCreateInfo<Title, Size> {
  pub title: Title,
  pub size: Size,
  pub color_mode: ColorMode,
  pub visibility: Visibility,
}

pub struct WindowBuilder<Title, Size> {
  create_info: WindowCreateInfo<Title, Size>,
}

impl WindowBuilder<MissingTitle, MissingSize> {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Default for WindowBuilder<MissingTitle, MissingSize> {
  fn default() -> Self {
    Self {
      create_info: WindowCreateInfo {
        title: MissingTitle,
        size: MissingSize,
        color_mode: ColorMode::Dark,
        visibility: Visibility::Shown,
      },
    }
  }
}

impl<Size> WindowBuilder<MissingTitle, Size> {
  pub fn with_title(self, title: &'static str) -> WindowBuilder<HasTitle, Size> {
    WindowBuilder {
      create_info: WindowCreateInfo {
        title: HasTitle(title),
        size: self.create_info.size,
        color_mode: self.create_info.color_mode,
        visibility: self.create_info.visibility,
      },
    }
  }
}

impl<Title> WindowBuilder<Title, MissingSize> {
  pub fn with_size(self, width: i32, height: i32) -> WindowBuilder<Title, HasSize> {
    WindowBuilder {
      create_info: WindowCreateInfo {
        title: self.create_info.title,
        size: HasSize { width, height },
        color_mode: self.create_info.color_mode,
        visibility: self.create_info.visibility,
      },
    }
  }
}

impl<Title, Size> WindowBuilder<Title, Size> {
  pub fn with_color_mode(self, color_mode: ColorMode) -> Self {
    Self {
      create_info: WindowCreateInfo {
        title: self.create_info.title,
        size: self.create_info.size,
        color_mode,
        visibility: self.create_info.visibility,
      },
    }
  }

  pub fn with_close_behavior(self, close_behavior: CloseBehavior) -> Self {
    Self {
      create_info: WindowCreateInfo {
        title: self.create_info.title,
        size: self.create_info.size,
        color_mode: self.create_info.color_mode,
        visibility: self.create_info.visibility,
      },
    }
  }

  pub fn with_visibility(self, visibility: Visibility) -> Self {
    Self {
      create_info: WindowCreateInfo {
        title: self.create_info.title,
        size: self.create_info.size,
        color_mode: self.create_info.color_mode,
        visibility,
      },
    }
  }
}

impl WindowBuilder<HasTitle, HasSize> {
  pub fn build(self) -> Result<Window, WindowError> {
    Window::new(self.create_info)
  }
}
