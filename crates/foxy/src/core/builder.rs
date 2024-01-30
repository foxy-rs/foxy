use foxy_utils::types::behavior::Polling;
use foxy_window::prelude::*;

use super::engine_loop::Framework;

#[derive(Default)]
pub enum DebugInfo {
  Shown,
  #[default]
  Hidden,
}

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
  pub debug_info: DebugInfo,
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
        debug_info: Default::default(),
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
        debug_info: self.create_info.debug_info,
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
        debug_info: self.create_info.debug_info,
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
        debug_info: self.create_info.debug_info,
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
        debug_info: self.create_info.debug_info,
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
        debug_info: self.create_info.debug_info,
      },
    }
  }

  pub fn with_debug_info(self, debug_info: DebugInfo) -> Self {
    Self {
      create_info: FoxyCreateInfo {
        title: self.create_info.title,
        size: self.create_info.size,
        color_mode: self.create_info.color_mode,
        close_behavior: self.create_info.close_behavior,
        polling_strategy: self.create_info.polling_strategy,
        debug_info,
      },
    }
  }
}

impl FoxyBuilder<HasTitle, HasSize> {
  pub fn build<'a>(self) -> anyhow::Result<Framework<'a>> {
    Framework::new(self.create_info)
  }

  pub fn build_unwrap<'a>(self) -> Framework<'a> {
    self.build().unwrap_or_else(|e| panic!("{e}"))
  }
}
