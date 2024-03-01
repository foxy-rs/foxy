use std::sync::Arc;

use ezwin::prelude::{Message, Window};
use foxy_time::Time;

use self::render_data::RenderData;
use crate::error::RendererError;

pub mod render_data;

// pub struct Egui {
//   context: egui::Context,
//   input: egui::RawInput,
//   state: egui_winit::State,
// }

pub struct Renderer {}

impl Renderer {
  pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
    Ok(Self {})
  }

  pub fn delete(&mut self) {}

  pub fn render(&mut self, render_time: Time, render_data: RenderData) -> Result<(), RendererError> {
    Ok(())
  }

  pub fn resize(&mut self) {}

  pub fn input(&mut self, _message: &Message) -> bool {
    false
  }
}
