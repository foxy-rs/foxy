use strum::EnumIter;
use winit::event::MouseButton;

#[derive(EnumIter, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u16)]
pub enum MouseCode {
  Left = 1,
  Right = 2,
  Middle = 3,
  Back = 4,
  Forward = 5,
  Extra(u16),
}

impl From<MouseButton> for MouseCode {
  fn from(value: MouseButton) -> Self {
    match value {
      MouseButton::Left => MouseCode::Left,
      MouseButton::Right => MouseCode::Right,
      MouseButton::Middle => MouseCode::Middle,
      MouseButton::Back => MouseCode::Back,
      MouseButton::Forward => MouseCode::Forward,
      MouseButton::Other(x) => MouseCode::Extra(x),
    }
  }
}
