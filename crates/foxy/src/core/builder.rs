use super::Foxy;
use foxy_types::window::Polling;
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
  pub polling_strategy: Polling,
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
        polling_strategy: Polling::Poll,
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
        polling_strategy: self.create_info.polling_strategy,
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
        polling_strategy: self.create_info.polling_strategy,
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
        polling_strategy: self.create_info.polling_strategy,
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
        polling_strategy: self.create_info.polling_strategy,
      },
    }
  }

  pub fn with_polling(self, message_behavior: Polling) -> Self {
    Self {
      create_info: FoxyCreateInfo {
        title: self.create_info.title,
        size: self.create_info.size,
        color_mode: self.create_info.color_mode,
        close_behavior: self.create_info.close_behavior,
        polling_strategy: message_behavior,
      },
    }
  }
}

impl FoxyBuilder<HasTitle, HasSize> {
  pub fn build<'a>(self) -> anyhow::Result<Foxy<'a>> {
    Foxy::new(self.create_info)
  }

  pub fn build_unwrap<'a>(self) -> Foxy<'a> {
    self.build().unwrap_or_else(|e| panic!("{e}"))
  }
}
