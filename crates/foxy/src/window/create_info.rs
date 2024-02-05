use winit::{
  dpi::{Position, Size},
  event_loop::{EventLoop, EventLoopBuilder},
  platform::windows::EventLoopBuilderExtWindows,
  window::{Fullscreen, Icon, Theme, Window, WindowBuilder, WindowButtons},
};

use crate::core::FoxyResult;

#[derive(Debug, Clone)]
pub struct WindowCreateInfo {
  pub inner_size: Option<Size>,
  pub min_inner_size: Option<Size>,
  pub max_inner_size: Option<Size>,
  pub position: Option<Position>,
  pub resizable: bool,
  pub enabled_buttons: WindowButtons,
  pub title: String,
  pub maximized: bool,
  pub visible: bool,
  pub transparent: bool,
  pub blur: bool,
  pub decorations: bool,
  pub window_icon: Option<Icon>,
  pub preferred_theme: Option<Theme>,
  pub resize_increments: Option<Size>,
  pub fullscreen: Option<Fullscreen>,
}

impl Default for WindowCreateInfo {
  fn default() -> Self {
    Self {
      inner_size: None,
      min_inner_size: None,
      max_inner_size: None,
      position: None,
      resizable: true,
      enabled_buttons: WindowButtons::all(),
      title: "Foxy Window".to_owned(),
      maximized: false,
      fullscreen: None,
      visible: false,
      transparent: false,
      blur: false,
      decorations: true,
      window_icon: None,
      preferred_theme: None,
      resize_increments: None,
    }
  }
}

impl WindowCreateInfo {
  pub fn create_window<T>(&self) -> FoxyResult<(EventLoop<T>, Window)> {
    let event_loop = EventLoopBuilder::<T>::with_user_event().with_any_thread(true).build()?;
    let mut builder = WindowBuilder::new();

    if let Some(size) = self.inner_size {
      builder = builder.with_inner_size(size);
    }

    if let Some(size) = self.min_inner_size {
      builder = builder.with_min_inner_size(size);
    }

    if let Some(size) = self.max_inner_size {
      builder = builder.with_max_inner_size(size);
    }

    if let Some(pos) = self.position {
      builder = builder.with_position(pos);
    }

    if let Some(s) = self.resize_increments {
      builder = builder.with_resize_increments(s);
    }

    builder = builder
      .with_resizable(self.resizable)
      .with_enabled_buttons(self.enabled_buttons)
      .with_title(self.title.clone())
      .with_maximized(self.maximized)
      .with_visible(self.visible)
      .with_transparent(self.transparent)
      .with_blur(self.blur)
      .with_decorations(self.decorations)
      .with_theme(self.preferred_theme)
      .with_fullscreen(self.fullscreen.clone());

    let window = builder.build(&event_loop)?;

    Ok((event_loop, window))
  }
}
