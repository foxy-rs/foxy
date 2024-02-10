pub mod key;
pub mod modifier;
pub mod mouse;
pub mod state;

use std::collections::HashMap;

use strum::IntoEnumIterator;
use winit::{
  event::{ElementState, MouseButton},
  keyboard::PhysicalKey,
};

use self::state::KeyState;
use crate::core::input::{key::KeyCode, mouse::MouseCode, state::ButtonState};

#[derive(Debug)]
pub struct Input {
  mouse_buttons: HashMap<MouseCode, ButtonState>,
  keys: HashMap<KeyCode, KeyState>,
  shift: ButtonState,
  ctrl: ButtonState,
  alt: ButtonState,
  win: ButtonState,
}

impl Input {
  pub fn new() -> Self {
    let mouse_buttons = {
      let mut map = HashMap::default();
      for code in MouseCode::iter() {
        map.insert(code, ButtonState::Released);
      }
      map
    };

    let keys = {
      let mut map = HashMap::default();
      for code in KeyCode::iter() {
        map.insert(code, KeyState::Released);
      }
      map
    };

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

  pub(crate) fn update_key_state(&mut self, keycode: PhysicalKey, state: ElementState, repeat: bool) {
    if let Some(key_state) = self.keys.get_mut(&keycode.into()) {
      *key_state = KeyState::from_winit(state, repeat);
    }
  }

  // MOUSE

  pub fn mouse(&self, code: MouseCode) -> ButtonState {
    self.mouse_buttons.get(&code).copied().unwrap_or(ButtonState::Released)
  }

  pub(crate) fn update_mouse_button_state(&mut self, button: MouseButton, state: ElementState) {
    if let Some(mouse_state) = self.mouse_buttons.get_mut(&button.into()) {
      *mouse_state = ButtonState::from_winit(state);
    }
  }

  // MODS

  pub fn shift(&self) -> ButtonState {
    self.shift
  }

  pub fn ctrl(&self) -> ButtonState {
    self.ctrl
  }

  pub fn alt(&self) -> ButtonState {
    self.alt
  }

  pub fn win(&self) -> ButtonState {
    self.win
  }

  pub(crate) fn update_modifiers_state(&mut self, modifiers: winit::event::Modifiers) {
    self.shift = if modifiers.state().shift_key() {
      ButtonState::Pressed
    } else {
      ButtonState::Released
    };

    self.ctrl = if modifiers.state().control_key() {
      ButtonState::Pressed
    } else {
      ButtonState::Released
    };

    self.alt = if modifiers.state().alt_key() {
      ButtonState::Pressed
    } else {
      ButtonState::Released
    };

    self.win = if modifiers.state().super_key() {
      ButtonState::Pressed
    } else {
      ButtonState::Released
    };
  }
}

impl Default for Input {
  fn default() -> Self {
    Self::new()
  }
}