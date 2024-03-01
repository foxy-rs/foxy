pub mod state;

use std::collections::HashMap;

use winit::{
  event::{ElementState, MouseButton},
  keyboard::KeyCode,
};

use self::state::KeyState;
use crate::core::input::state::MouseState;

#[derive(Debug)]
pub struct Input {
  mouse_buttons: HashMap<MouseButton, MouseState>,
  keys: HashMap<KeyCode, KeyState>,
  shift: MouseState,
  ctrl: MouseState,
  alt: MouseState,
  win: MouseState,
}

impl Input {
  pub fn new() -> Self {
    let mouse_buttons = HashMap::default();

    let keys = HashMap::default();

    Self {
      mouse_buttons,
      keys,
      shift: Default::default(),
      ctrl: Default::default(),
      alt: Default::default(),
      win: Default::default(),
    }
  }

  // KEYBOARD

  pub fn key(&self, code: KeyCode) -> KeyState {
    self.keys.get(&code).copied().unwrap_or(KeyState::Released)
  }

  pub(crate) fn update_key_state(&mut self, keycode: KeyCode, state: ElementState, repeat: bool) {
    let state = KeyState::from_winit(state, repeat);
    self.keys.insert(keycode, state);
  }

  // MOUSE

  pub fn mouse(&self, code: MouseButton) -> MouseState {
    self.mouse_buttons.get(&code).copied().unwrap_or(MouseState::Released)
  }

  pub(crate) fn update_mouse_button_state(&mut self, button: MouseButton, state: ElementState) {
    let state = MouseState::from_winit(state);
    self.mouse_buttons.insert(button, state);
  }

  // MODS

  pub fn shift(&self) -> MouseState {
    self.shift
  }

  pub fn ctrl(&self) -> MouseState {
    self.ctrl
  }

  pub fn alt(&self) -> MouseState {
    self.alt
  }

  pub fn win(&self) -> MouseState {
    self.win
  }

  pub(crate) fn update_modifiers_state(&mut self, modifiers: winit::event::Modifiers) {
    self.shift = if modifiers.state().shift_key() {
      MouseState::Pressed
    } else {
      MouseState::Released
    };

    self.ctrl = if modifiers.state().control_key() {
      MouseState::Pressed
    } else {
      MouseState::Released
    };

    self.alt = if modifiers.state().alt_key() {
      MouseState::Pressed
    } else {
      MouseState::Released
    };

    self.win = if modifiers.state().super_key() {
      MouseState::Pressed
    } else {
      MouseState::Released
    };
  }
}

impl Default for Input {
  fn default() -> Self {
    Self::new()
  }
}
