use winit::event::KeyEvent;

use super::input::{
  key::KeyCode,
  mouse::MouseCode,
  state::{ButtonState, KeyState},
};

#[derive(Debug, Clone, PartialEq)]
pub enum FoxyEvent {
  None,
  Window(WindowEvent),
  Input(InputEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub enum WindowEvent {
  Moved,
  Resized,
  Rescaled,
  Unmapped(winit::event::WindowEvent),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
  Mouse(MouseCode, ButtonState),
  Keyboard(KeyCode, KeyState),
  Cursor,
  Scroll,
}

impl From<Option<winit::event::WindowEvent>> for FoxyEvent {
  fn from(value: Option<winit::event::WindowEvent>) -> Self {
    let Some(value) = value else {
      return Self::None;
    };

    match value {
      winit::event::WindowEvent::Resized(_) => Self::Window(WindowEvent::Resized),
      winit::event::WindowEvent::Moved(_) => Self::Window(WindowEvent::Moved),
      winit::event::WindowEvent::KeyboardInput {
        event: KeyEvent {
          physical_key,
          state,
          repeat,
          ..
        },
        ..
      } => FoxyEvent::Input(InputEvent::Keyboard(physical_key.into(), KeyState::from_winit(state, repeat))),
      winit::event::WindowEvent::CursorMoved { .. } => Self::Input(InputEvent::Cursor),
      winit::event::WindowEvent::MouseWheel { .. } => Self::Input(InputEvent::Scroll),
      winit::event::WindowEvent::MouseInput { state, button, .. } => {
        FoxyEvent::Input(InputEvent::Mouse(button.into(), ButtonState::from_winit(state)))
      }
      winit::event::WindowEvent::ScaleFactorChanged { .. } => Self::Window(WindowEvent::Rescaled),
      _ => Self::Window(WindowEvent::Unmapped(value)),
    }
  }
}