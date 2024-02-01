use enumflags2::BitFlags;
use foxy_utils::log::LogErr;
use windows::Win32::{
  Foundation::{HINSTANCE, HWND, LPARAM, RECT, WPARAM},
  System::SystemServices::{MK_LBUTTON, MK_MBUTTON, MK_RBUTTON, MK_XBUTTON1, MK_XBUTTON2, MODIFIERKEYS_FLAGS},
  UI::{
    Input::KeyboardAndMouse::{MapVirtualKeyW, MAPVK_VSC_TO_VK_EX, VIRTUAL_KEY},
    WindowsAndMessaging::{self, *},
  },
};

use super::input::{
  button::{ButtonState, KeyState},
  keyboard::KeyCode,
  modifier::Modifiers,
  mouse::MouseCode,
};
use crate::{hiword, lobyte, loword};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum WindowMessage {
  #[default]
  None,
  State(StateMessage),
  Keyboard(KeyboardMessage),
  Mouse(MouseMessage),
  Other {
    hwnd: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
  },
  ExitLoop,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum StateMessage {
  Ready { hwnd: HWND, hinstance: HINSTANCE },
  Resized { window_rect: RECT, client_rect: RECT },
  Moved,
  CloseRequested,
  Closing,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum KeyboardMessage {
  Key {
    key_code: KeyCode,
    state: KeyState,
    scan_code: u16,
    is_extended_key: bool,
  },
  Modifiers {
    mods: BitFlags<Modifiers>,
  },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MouseMessage {
  Button {
    mouse_code: MouseCode,
    state: ButtonState,
    position: (u16, u16),
    is_double_click: bool,
  },
  Cursor,
  Scroll,
}

impl WindowMessage {
  pub fn new(hwnd: HWND, message: u32, w_param: WPARAM, l_param: LPARAM) -> Self {
    match message {
      WindowsAndMessaging::WM_CLOSE => WindowMessage::State(StateMessage::CloseRequested),
      WindowsAndMessaging::WM_DESTROY => WindowMessage::State(StateMessage::Closing),
      WindowsAndMessaging::WM_QUIT => WindowMessage::ExitLoop,
      WindowsAndMessaging::WM_MOVING => WindowMessage::State(StateMessage::Moved),
      // WindowsAndMessaging::WM_SIZE => // this is for things like maximized, minimized, etc.
      WindowsAndMessaging::WM_SIZING => {
        let mut window_rect = RECT::default();
        let _ = unsafe { GetWindowRect(hwnd, std::ptr::addr_of_mut!(window_rect)) }.log_error();
        let mut client_rect = RECT::default();
        let _ = unsafe { GetClientRect(hwnd, std::ptr::addr_of_mut!(client_rect)) }.log_error();
        WindowMessage::State(StateMessage::Resized {
          window_rect,
          client_rect,
        })
      }
      msg if (WindowsAndMessaging::WM_KEYFIRST..=WindowsAndMessaging::WM_KEYLAST).contains(&msg) => {
        Self::new_keyboard_message(l_param)
      }
      WindowsAndMessaging::WM_LBUTTONDBLCLK
      | WindowsAndMessaging::WM_RBUTTONDBLCLK
      | WindowsAndMessaging::WM_MBUTTONDBLCLK
      | WindowsAndMessaging::WM_XBUTTONDBLCLK
      | WindowsAndMessaging::WM_LBUTTONDOWN
      | WindowsAndMessaging::WM_RBUTTONDOWN
      | WindowsAndMessaging::WM_MBUTTONDOWN
      | WindowsAndMessaging::WM_XBUTTONDOWN
      | WindowsAndMessaging::WM_LBUTTONUP
      | WindowsAndMessaging::WM_RBUTTONUP
      | WindowsAndMessaging::WM_MBUTTONUP
      | WindowsAndMessaging::WM_XBUTTONUP => Self::new_mouse_button_message(message, w_param, l_param),
      WindowsAndMessaging::WM_MOUSEMOVE => WindowMessage::Mouse(MouseMessage::Cursor),
      WindowsAndMessaging::WM_MOUSEWHEEL | WindowsAndMessaging::WM_MOUSEHWHEEL => {
        WindowMessage::Mouse(MouseMessage::Scroll)
      }
      _ => WindowMessage::Other {
        hwnd,
        message,
        w_param,
        l_param,
      },
    }
  }

  fn new_keyboard_message(l_param: LPARAM) -> WindowMessage {
    let flags = hiword(unsafe { std::mem::transmute::<i32, u32>(l_param.0 as i32) });

    let is_extended_key = (flags & KF_EXTENDED as u16) == KF_EXTENDED as u16;

    let mut scan_code = lobyte(flags) as u16;

    let key_code: KeyCode = {
      let extended_scan_code = u16::from_le_bytes([scan_code as u8, 0xE0]);
      let extended_virtual_keycode =
        VIRTUAL_KEY(loword(unsafe { MapVirtualKeyW(extended_scan_code as u32, MAPVK_VSC_TO_VK_EX) }));

      let virtual_keycode = if extended_virtual_keycode != VIRTUAL_KEY(0) && is_extended_key {
        scan_code = extended_scan_code;
        extended_virtual_keycode
      } else {
        VIRTUAL_KEY(loword(unsafe { MapVirtualKeyW(scan_code as u32, MAPVK_VSC_TO_VK_EX) }))
      };

      virtual_keycode.into()
    };

    let state = {
      let was_key_down = (flags & KF_REPEAT as u16) == KF_REPEAT as u16;
      let repeat_count = loword(l_param.0 as u32);
      let is_key_up = (flags & KF_UP as u16) == KF_UP as u16;

      if is_key_up {
        KeyState::Released
      } else if was_key_down {
        KeyState::Held { repeat_count }
      } else {
        KeyState::Pressed
      }
    };

    WindowMessage::Keyboard(KeyboardMessage::Key {
      key_code,
      state,
      scan_code,
      is_extended_key,
    })
  }

  fn new_mouse_button_message(message: u32, w_param: WPARAM, l_param: LPARAM) -> WindowMessage {
    let flags = w_param.0 as u32;

    let mouse_code: MouseCode = {
      match message {
        WindowsAndMessaging::WM_LBUTTONDBLCLK
        | WindowsAndMessaging::WM_LBUTTONDOWN
        | WindowsAndMessaging::WM_LBUTTONUP => MouseCode::Left,
        WindowsAndMessaging::WM_MBUTTONDBLCLK
        | WindowsAndMessaging::WM_MBUTTONDOWN
        | WindowsAndMessaging::WM_MBUTTONUP => MouseCode::Middle,
        WindowsAndMessaging::WM_RBUTTONDBLCLK
        | WindowsAndMessaging::WM_RBUTTONDOWN
        | WindowsAndMessaging::WM_RBUTTONUP => MouseCode::Right,
        WindowsAndMessaging::WM_XBUTTONDBLCLK
        | WindowsAndMessaging::WM_XBUTTONDOWN
        | WindowsAndMessaging::WM_XBUTTONUP => {
          let hiflags = hiword(flags);
          if (hiflags & XBUTTON1) == XBUTTON1 {
            MouseCode::Back
          } else {
            MouseCode::Forward
          }
        }
        _ => MouseCode::Unknown,
      }
    };

    let is_double_click = matches!(
      message,
      WindowsAndMessaging::WM_LBUTTONDBLCLK
        | WindowsAndMessaging::WM_MBUTTONDBLCLK
        | WindowsAndMessaging::WM_RBUTTONDBLCLK
        | WindowsAndMessaging::WM_XBUTTONDBLCLK
    );

    let state = {
      let mod_flags = MODIFIERKEYS_FLAGS(flags);
      let is_l_down = (mod_flags & MK_LBUTTON) == MK_LBUTTON;
      let is_m_down = (mod_flags & MK_MBUTTON) == MK_MBUTTON;
      let is_r_down = (mod_flags & MK_RBUTTON) == MK_RBUTTON;
      let is_x1_down = (mod_flags & MK_XBUTTON1) == MK_XBUTTON1;
      let is_x2_down = (mod_flags & MK_XBUTTON2) == MK_XBUTTON2;

      let is_down = match message {
        WindowsAndMessaging::WM_LBUTTONDBLCLK | WindowsAndMessaging::WM_LBUTTONDOWN if is_l_down => true,
        WindowsAndMessaging::WM_MBUTTONDBLCLK | WindowsAndMessaging::WM_MBUTTONDOWN if is_m_down => true,
        WindowsAndMessaging::WM_RBUTTONDBLCLK | WindowsAndMessaging::WM_RBUTTONDOWN if is_r_down => true,
        WindowsAndMessaging::WM_XBUTTONDBLCLK | WindowsAndMessaging::WM_XBUTTONDOWN if is_x1_down || is_x2_down => true,
        _ => false,
      };

      if is_down {
        ButtonState::Pressed
      } else {
        ButtonState::Released
      }
    };

    let position = (loword(l_param.0 as u32), hiword(l_param.0 as u32));

    WindowMessage::Mouse(MouseMessage::Button {
      mouse_code,
      state,
      position,
      is_double_click,
    })
  }
}
