use ezwin::prelude::*;

use super::App;

pub struct HasTitle(pub &'static str);
pub struct MissingTitle;

pub struct HasSize {
  pub width: i32,
  pub height: i32,
}
pub struct MissingSize;

pub struct AppCreateInfo<Title, Size> {
  pub title: Title,
  pub size: Size,
  pub color_mode: ColorMode,
  pub close_behavior: CloseBehavior,
}

pub struct AppBuilder<Title, Size> {
  create_info: AppCreateInfo<Title, Size>,
}

impl AppBuilder<MissingTitle, MissingSize> {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Default for AppBuilder<MissingTitle, MissingSize> {
  fn default() -> Self {
    Self {
      create_info: AppCreateInfo {
        title: MissingTitle,
        size: MissingSize,
        color_mode: ColorMode::Dark,
        close_behavior: CloseBehavior::Default,
      },
    }
  }
}

impl<Size> AppBuilder<MissingTitle, Size> {
  pub fn with_title(self, title: &'static str) -> AppBuilder<HasTitle, Size> {
    AppBuilder {
      create_info: AppCreateInfo {
        title: HasTitle(title),
        size: self.create_info.size,
        color_mode: self.create_info.color_mode,
        close_behavior: self.create_info.close_behavior,
      },
    }
  }
}

impl<Title> AppBuilder<Title, MissingSize> {
  pub fn with_size(self, width: i32, height: i32) -> AppBuilder<Title, HasSize> {
    AppBuilder {
      create_info: AppCreateInfo {
        title: self.create_info.title,
        size: HasSize { width, height },
        color_mode: self.create_info.color_mode,
        close_behavior: self.create_info.close_behavior,
      },
    }
  }
}

impl<Title, Size> AppBuilder<Title, Size> {
  pub fn with_dark_mode(self, color_mode: ColorMode) -> Self {
    Self {
      create_info: AppCreateInfo {
        title: self.create_info.title,
        size: self.create_info.size,
        color_mode,
        close_behavior: self.create_info.close_behavior,
      },
    }
  }

  pub fn with_close_behavior(self, close_behavior: CloseBehavior) -> Self {
    Self {
      create_info: AppCreateInfo {
        title: self.create_info.title,
        size: self.create_info.size,
        color_mode: self.create_info.color_mode,
        close_behavior,
      },
    }
  }
}

impl AppBuilder<HasTitle, HasSize> {
  pub fn run(self) {
    if let Ok(app) = App::new(self.create_info) {
      app.run();
    }
  }
}
