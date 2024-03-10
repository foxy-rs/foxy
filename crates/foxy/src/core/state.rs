use std::{sync::Arc, time::Duration};

use egui::{epaint::Shadow, RawInput, Rounding, Visuals};
use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_time::{timer::Timer, Time, TimeSettings};
use tracing::error;
use witer::prelude::*;

use super::{builder::DebugInfo, FoxyResult};

pub struct Foxy {
  pub(crate) time: Time,
  pub(crate) window: Arc<Window>,
  pub(crate) egui_context: egui::Context,

  pub(crate) renderer: Renderer,
  pub(crate) render_data: RenderData,

  pub(crate) debug_info: DebugInfo,
  pub(crate) fps_timer: Timer,
  frame_count: u32,
  is_revealed: bool,
}

impl Foxy {
  pub fn new(window: Arc<Window>, time_settings: TimeSettings, debug_info: DebugInfo) -> FoxyResult<Self> {
    let egui_context = egui::Context::default();

    const BORDER_RADIUS: f32 = 6.0;

    let visuals = Visuals {
      window_rounding: Rounding::same(BORDER_RADIUS),
      menu_rounding: Rounding::same(BORDER_RADIUS),
      window_shadow: Shadow::NONE,
      ..Default::default()
    };

    egui_context.set_visuals(visuals);

    let time = time_settings.build();
    let renderer = Renderer::new(window.clone())?;

    Ok(Self {
      time,
      window,
      egui_context,
      renderer,
      render_data: RenderData::default(),
      debug_info,
      fps_timer: Timer::new(),
      frame_count: 0,
      is_revealed: false,
    })
  }

  pub fn delta_time(&self) -> Duration {
    *self.time.delta()
  }

  pub fn average_delta_time(&self) -> Duration {
    *self.time.average_delta()
  }

  pub fn window(&self) -> &Arc<Window> {
    &self.window
  }

  pub fn key(&self, key: Key) -> KeyState {
    self.window.key(key)
  }

  pub fn mouse(&self, mouse: Mouse) -> ButtonState {
    self.window.mouse(mouse)
  }

  pub fn shift(&self) -> ButtonState {
    self.window.shift()
  }

  pub fn ctrl(&self) -> ButtonState {
    self.window.ctrl()
  }

  pub fn alt(&self) -> ButtonState {
    self.window.alt()
  }

  pub fn win(&self) -> ButtonState {
    self.window.win()
  }

  pub fn take_egui_raw_input(&self) -> RawInput {
    RawInput::default()
  }

  pub(crate) fn render(&mut self) -> bool {
    if let Err(error) = self.renderer.render(&self.time, &self.render_data) {
      error!("`{error}` Aborting...");
      return false;
    }

    match (self.is_revealed, self.frame_count) {
      (false, 10) => {
        self.window.set_visibility(Visibility::Shown);
        self.is_revealed = true;
      }
      (false, _) => self.frame_count = self.frame_count.wrapping_add(1),
      _ => (),
    };

    if self.fps_timer.has_elapsed(Duration::from_millis(200)) {
      if let DebugInfo::Shown = self.debug_info {
        let ft = self.time.average_delta_secs();
        self
          .window
          .set_subtitle(format!(" | {:^5.4} s | {:>5.0} FPS", ft, 1.0 / ft));
      }
    }

    true
  }
}
