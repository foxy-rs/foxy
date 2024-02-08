#![deny(unsafe_op_in_unsafe_fn)]

use std::{collections::HashSet, mem::ManuallyDrop, sync::Arc, time::Duration};

use foxy_utils::{log::LogErr, time::Time};
use tracing::*;
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use crate::{error::RendererError, renderer::render_data::RenderData};

use self::instance::FoxyInstance;

pub mod instance;
pub mod error;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum ValidationStatus {
  Enabled,
  #[default]
  Disabled,
}

pub struct Vulkan {
  instance: FoxyInstance,
  window: Arc<Window>,
}

impl Vulkan {
  pub fn new(window: Arc<Window>) -> Result<Self, RendererError> {
    trace!("Initializing Vulkan");

    // init vulkan
    let instance = FoxyInstance::new(&window)?;

    Ok(Self {
      window,
      instance,
    })
  }

  pub fn render(&mut self, render_time: Time, _render_data: RenderData) -> Result<bool, RendererError> {
    Ok(false)
  }

  pub fn resize(&mut self) {}

  pub fn input(&mut self, event: &WindowEvent) -> bool {
    false
  }
}
