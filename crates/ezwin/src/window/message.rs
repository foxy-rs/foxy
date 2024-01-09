use enumflags2::BitFlags;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    System::SystemServices::{
        MK_LBUTTON, MK_MBUTTON, MK_RBUTTON, MK_XBUTTON1, MK_XBUTTON2, MODIFIERKEYS_FLAGS,
    },
    UI::{
        Input::KeyboardAndMouse::{MapVirtualKeyW, MAPVK_VSC_TO_VK_EX, VIRTUAL_KEY},
        WindowsAndMessaging::{self, *},
    },
};

use crate::{hiword, lobyte, loword};

use super::input::{
    button::{ButtonState, KeyState},
    keyboard::KeyCode,
    modifier::Modifiers,
    mouse::MouseCode,
};

// #[derive(Debug, Display, PartialEq, Eq)]
// pub enum Message {
//     Empty,
//     Window(WindowMessage),
//     Keyboard(KeyboardMessage),
//     Mouse(MouseMessage),
//     Exit,
// }

#[derive(Debug, PartialEq, Eq)]
pub enum WindowMessage {
    Empty,
    Ready { hwnd: HWND },
    CloseRequested,
    Keyboard(KeyboardMessage),
    Mouse(MouseMessage),
    // Closed,
    Exit,
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
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
    pub fn new(_hwnd: HWND, message: u32, w_param: WPARAM, l_param: LPARAM) -> Self {
        match message {
            WindowsAndMessaging::WM_CLOSE => WindowMessage::CloseRequested,
            // WindowsAndMessaging::WM_NCDESTROY => WindowMessage::Closed,
            WindowsAndMessaging::WM_QUIT => WindowMessage::Exit,
            WindowsAndMessaging::WM_KEYDOWN
            | WindowsAndMessaging::WM_KEYUP
            | WindowsAndMessaging::WM_SYSKEYDOWN
            | WindowsAndMessaging::WM_SYSKEYUP => Self::new_keyboard_message(l_param),
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
            | WindowsAndMessaging::WM_XBUTTONUP => {
                Self::new_mouse_button_message(message, w_param, l_param)
            }
            WindowsAndMessaging::WM_MOUSEMOVE => WindowMessage::Mouse(MouseMessage::Cursor),
            WindowsAndMessaging::WM_MOUSEWHEEL | WindowsAndMessaging::WM_MOUSEHWHEEL => {
                WindowMessage::Mouse(MouseMessage::Scroll)
            }
            _ => WindowMessage::Empty,
        }
    }

    fn new_keyboard_message(l_param: LPARAM) -> WindowMessage {
        let flags = hiword(unsafe { std::mem::transmute::<i32, u32>(l_param.0 as i32) });
        // debug!(
        //     "\nMessage: {:#034b}\nwParam:  {:#0wparam_width$b}\nlParam:  {:#0lparam_width$b}\nvk:      {:#018b}\nflags:   {:#018b}",
        //     msg.message,
        //     msg.wParam.0,
        //     msg.lParam.0,
        //     loword(msg.wParam.0 as u32),
        //     flags,
        //     wparam_width = std::mem::size_of::<WPARAM>() * 8 + 2,
        //     lparam_width = std::mem::size_of::<LPARAM>() * 8 + 2,
        // );

        let is_extended_key = (flags & KF_EXTENDED as u16) == KF_EXTENDED as u16;

        let mut scan_code = lobyte(flags) as u16;

        let key_code: KeyCode = {
            let extended_scan_code = u16::from_le_bytes([scan_code as u8, 0xE0]);
            let extended_virtual_keycode = VIRTUAL_KEY(loword(unsafe {
                MapVirtualKeyW(extended_scan_code as u32, MAPVK_VSC_TO_VK_EX)
            }));

            let virtual_keycode = if extended_virtual_keycode != VIRTUAL_KEY(0) && is_extended_key {
                scan_code = extended_scan_code;
                extended_virtual_keycode
            } else {
                VIRTUAL_KEY(loword(unsafe {
                    MapVirtualKeyW(scan_code as u32, MAPVK_VSC_TO_VK_EX)
                }))
            };

            virtual_keycode.into()
        };

        // key_code = match key_code {
        //     KeyCode::Enter if is_extended_key => {
        //         KeyCode::NumEnter
        //     },
        //     KeyCode::Plus if is_extended_key => {
        //         KeyCode::NumPlus
        //     },
        //     KeyCode::Comma if is_extended_key => {
        //         KeyCode::NumComma
        //     },
        //     _ => key_code
        // };

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
        //debug!("\nMessage: {:#032b}\nwParam:  {:#032b}\nlParam:  {:#032b}", msg.message, msg.wParam.0, msg.lParam.0);
        let position = (loword(l_param.0 as u32), hiword(l_param.0 as u32));
        let flags = w_param.0 as u32;
        let mod_flags = MODIFIERKEYS_FLAGS(flags);
        let is_l_down = (mod_flags & MK_LBUTTON) == MK_LBUTTON;
        let is_m_down = (mod_flags & MK_MBUTTON) == MK_MBUTTON;
        let is_r_down = (mod_flags & MK_RBUTTON) == MK_RBUTTON;
        let is_x1_down = (mod_flags & MK_XBUTTON1) == MK_XBUTTON1;
        let is_x2_down = (mod_flags & MK_XBUTTON2) == MK_XBUTTON2;

        let mouse_code: MouseCode = {
            let virtual_keycode = VIRTUAL_KEY(loword(w_param.0 as u32));
            // match msg.message {
            //     WindowsAndMessaging::WM_LBUTTONDBLCLK | WindowsAndMessaging::WM_LBUTTONDOWN | WindowsAndMessaging::WM_LBUTTONUP => {
            //         MouseCode::Left
            //     }
            //     WindowsAndMessaging::WM_MBUTTONDBLCLK | WindowsAndMessaging::WM_MBUTTONDOWN | WindowsAndMessaging::WM_RBUTTONUP => {
            //         MouseCode::Middle
            //     }
            //     WindowsAndMessaging::WM_RBUTTONDBLCLK | WindowsAndMessaging::WM_RBUTTONDOWN | WindowsAndMessaging::WM_MBUTTONUP => {
            //         MouseCode::Right
            //     }
            //     WindowsAndMessaging::WM_XBUTTONDBLCLK | WindowsAndMessaging::WM_XBUTTONDOWN | WindowsAndMessaging::WM_XBUTTONUP => {
            //         MouseCode::Left
            //     }
            //     WindowsAndMessaging::WM_XBUTTONDBLCLK | WindowsAndMessaging::WM_XBUTTONDOWN | WindowsAndMessaging::WM_XBUTTONUP => {
            //         MouseCode::Left
            //     }
            //     _ => false
            // };

            virtual_keycode.into()
        };
        // debug!("{:?}", mouse_code);

        let is_double_click = matches!(
            message,
            WindowsAndMessaging::WM_LBUTTONDBLCLK
                | WindowsAndMessaging::WM_MBUTTONDBLCLK
                | WindowsAndMessaging::WM_RBUTTONDBLCLK
                | WindowsAndMessaging::WM_XBUTTONDBLCLK
        );

        let state = {
            let is_down = match message {
                WindowsAndMessaging::WM_LBUTTONDBLCLK | WindowsAndMessaging::WM_LBUTTONDOWN
                    if is_l_down =>
                {
                    true
                }
                WindowsAndMessaging::WM_MBUTTONDBLCLK | WindowsAndMessaging::WM_MBUTTONDOWN
                    if is_m_down =>
                {
                    true
                }
                WindowsAndMessaging::WM_RBUTTONDBLCLK | WindowsAndMessaging::WM_RBUTTONDOWN
                    if is_r_down =>
                {
                    true
                }
                WindowsAndMessaging::WM_XBUTTONDBLCLK | WindowsAndMessaging::WM_XBUTTONDOWN
                    if is_x1_down || is_x2_down =>
                {
                    true
                }
                _ => false,
            };

            if is_down {
                ButtonState::Released
            } else {
                ButtonState::Pressed
            }
        };

        WindowMessage::Mouse(MouseMessage::Button {
            mouse_code,
            state,
            position,
            is_double_click,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppMessage {
    Empty,
    Exit,
}
