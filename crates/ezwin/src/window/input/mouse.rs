use strum::EnumIter;

#[derive(EnumIter, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[repr(u16)]
pub enum MouseCode {
  Unknown = 0,
  Left = 1,
  Right = 2,
  Middle = 3,
  Back = 4,
  Forward = 5,
}

// impl From<VIRTUAL_KEY> for MouseCode {
//     fn from(value: VIRTUAL_KEY) -> Self {
//         match value {
//             KeyboardAndMouse::VK_LBUTTON => MouseCode::Left,
//             KeyboardAndMouse::VK_RBUTTON => MouseCode::Right,
//             KeyboardAndMouse::VK_MBUTTON => MouseCode::Middle,
//             KeyboardAndMouse::VK_XBUTTON1 => MouseCode::Back,
//             KeyboardAndMouse::VK_XBUTTON2 => MouseCode::Forward,
//             _ => MouseCode::Unknown,
//         }
//     }
// }
