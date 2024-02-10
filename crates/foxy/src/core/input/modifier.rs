// pub enum  Modifiers {
//
// }

use strum::EnumIter;
use winit::keyboard::ModifiersState;

#[derive(EnumIter, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Modifier {
  Shift,
  Ctrl,
  Alt,
  Super,
}

impl From<Modifier> for ModifiersState {
  fn from(val: Modifier) -> Self {
    match val {
      Modifier::Shift => ModifiersState::SHIFT,
      Modifier::Ctrl => ModifiersState::CONTROL,
      Modifier::Alt => ModifiersState::ALT,
      Modifier::Super => ModifiersState::SUPER,
    }
  }
}