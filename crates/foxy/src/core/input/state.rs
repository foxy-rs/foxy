use winit::event::ElementState;

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub enum MouseState {
  #[default]
  Released,
  Pressed,
}

impl MouseState {
  pub fn from_winit(state: ElementState) -> Self {
    match state {
      ElementState::Pressed => Self::Pressed,
      ElementState::Released => Self::Released,
    }
  }

  pub fn is_pressed(self) -> bool {
    self == MouseState::Pressed
  }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Hash)]
pub enum KeyState {
  #[default]
  Released,
  Pressed,
  Held,
}

impl KeyState {
  pub fn from_winit(state: ElementState, repeat: bool) -> Self {
    match (state, repeat) {
      (ElementState::Pressed, false) => Self::Pressed,
      (ElementState::Pressed, true) => Self::Held,
      (ElementState::Released, _) => Self::Released,
    }
  }

  pub fn is_pressed(self) -> bool {
    self == KeyState::Pressed
  }

  pub fn is_held(self) -> bool {
    // this covers the first moments of the keypress as well
    self != KeyState::Released
  }
}
