use std::sync::Arc;

use foxy_utils::time::Time;
use glium::glutin::display::{Display, DisplayApiPreference};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{event::WindowEvent, window::Window};

use self::render_data::RenderData;
use crate::error::RendererError;

pub mod render_data;

pub struct Renderer {
  window: Arc<Window>,
  display: Display,

  is_dirty: bool,
}

impl Renderer {
  pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
    let display = unsafe {
      glium::glutin::display::Display::new(
        window.raw_display_handle(),
        DisplayApiPreference::WglThenEgl(Some(window.raw_window_handle())),
      )
    }?;

    Ok(Self {
      window,
      display,
      is_dirty: false,
    })
  }

  pub fn window(&self) -> &Window {
    self.window.as_ref()
  }

  pub fn refresh(&mut self) {
    self.is_dirty = true;
  }

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    false
  }

  pub fn draw(&mut self, render_time: Time, render_data: RenderData) -> Result<(), RendererError> {
    Ok(())
  }
}
