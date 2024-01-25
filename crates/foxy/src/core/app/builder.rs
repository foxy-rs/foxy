use super::Foxy;
use foxy_window::prelude::*;

pub struct HasTitle(pub &'static str);
pub struct MissingTitle;

pub struct HasSize {
  pub width: i32,
  pub height: i32,
}
pub struct MissingSize;

pub struct FoxyCreateInfo<Title, Size> {
  pub title: Title,
  pub size: Size,
  pub color_mode: ColorMode,
  pub close_behavior: CloseBehavior,
}

pub struct FoxyBuilder<Title, Size> {
  create_info: FoxyCreateInfo<Title, Size>,
}

impl FoxyBuilder<MissingTitle, MissingSize> {
  pub fn new() -> Self {
    Self::default()
  }
}

impl Default for FoxyBuilder<MissingTitle, MissingSize> {
  fn default() -> Self {
    Self {
      create_info: FoxyCreateInfo {
        title: MissingTitle,
        size: MissingSize,
        color_mode: ColorMode::Dark,
        close_behavior: CloseBehavior::Default,
      },
    }
  }
}

impl<Size> FoxyBuilder<MissingTitle, Size> {
  pub fn with_title(self, title: &'static str) -> FoxyBuilder<HasTitle, Size> {
    FoxyBuilder {
      create_info: FoxyCreateInfo {
        title: HasTitle(title),
        size: self.create_info.size,
        color_mode: self.create_info.color_mode,
        close_behavior: self.create_info.close_behavior,
      },
    }
  }
}

impl<Title> FoxyBuilder<Title, MissingSize> {
  pub fn with_size(self, width: i32, height: i32) -> FoxyBuilder<Title, HasSize> {
    FoxyBuilder {
      create_info: FoxyCreateInfo {
        title: self.create_info.title,
        size: HasSize { width, height },
        color_mode: self.create_info.color_mode,
        close_behavior: self.create_info.close_behavior,
      },
    }
  }
}

impl<Title, Size> FoxyBuilder<Title, Size> {
  pub fn with_dark_mode(self, color_mode: ColorMode) -> Self {
    Self {
      create_info: FoxyCreateInfo {
        title: self.create_info.title,
        size: self.create_info.size,
        color_mode,
        close_behavior: self.create_info.close_behavior,
      },
    }
  }

  pub fn with_close_behavior(self, close_behavior: CloseBehavior) -> Self {
    Self {
      create_info: FoxyCreateInfo {
        title: self.create_info.title,
        size: self.create_info.size,
        color_mode: self.create_info.color_mode,
        close_behavior,
      },
    }
  }
}

impl FoxyBuilder<HasTitle, HasSize> {
  pub fn build(self) -> anyhow::Result<Foxy> {
    Foxy::new(self.create_info)
  }
}
