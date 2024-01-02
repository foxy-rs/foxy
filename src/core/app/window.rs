// Reference for multithreaded input processing:
//   * https://www.jendrikillner.com/post/rust-game-part-3/
//   * https://github.com/jendrikillner/RustMatch3/blob/rust-game-part-3/

use std::sync::mpsc::{Receiver, Sender};

use tracing::*;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
};

use self::message::Message;

pub mod message;
pub mod keyboard;

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum Visibility {
    Shown,
    Hidden,
}

extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            LRESULT::default()
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

#[allow(unused)]
pub struct AppWindow {
    hwnd: HWND,
    message_receiver: Receiver<Message>,
}

impl AppWindow {
    pub const THREAD_ID: &'static str = "window";

    pub fn new(width: i32, height: i32, title: &str, dark_mode: bool) -> anyhow::Result<Self> {
        let title = HSTRING::from(title);

        let (message_sender, message_receiver) = std::sync::mpsc::channel();

        std::thread::Builder::new()
            .name(Self::THREAD_ID.into())
            .spawn(move || -> anyhow::Result<()> {
                let instance = unsafe { GetModuleHandleW(None)? };
                debug_assert_ne!(instance.0, 0);

                let window_class = title.clone();

                let wc = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    style: CS_VREDRAW | CS_HREDRAW,
                    lpfnWndProc: Some(wndproc),
                    hInstance: instance.into(),
                    hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
                    // hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
                    lpszClassName: PCWSTR(window_class.as_ptr()),
                    ..Default::default()
                };

                unsafe {
                    let atom = RegisterClassExW(&wc);
                    debug_assert_ne!(atom, 0);
                }

                let hwnd = unsafe {
                    CreateWindowExW(
                        WINDOW_EX_STYLE::default(),
                        &window_class,
                        &title,
                        WS_OVERLAPPEDWINDOW,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        width,
                        height,
                        None,
                        None,
                        instance,
                        None,
                    )
                };

                let dark_mode = BOOL::from(dark_mode);
                unsafe {
                    DwmSetWindowAttribute(
                        hwnd,
                        DWMWA_USE_IMMERSIVE_DARK_MODE,
                        std::ptr::addr_of!(dark_mode) as *const std::ffi::c_void,
                        std::mem::size_of::<BOOL>() as u32,
                    )
                }?;

                Self::message_pump(hwnd, &message_sender)
            })?;

        if let Message::WindowOpened { hwnd } = message_receiver.recv()? {
            Ok(Self {
                hwnd,
                message_receiver,
            })
        } else {
            Err(anyhow::format_err!("Invalid message"))
        }
    }

    fn message_pump(hwnd: HWND, message_sender: &Sender<Message>) -> anyhow::Result<()> {
        message_sender.send(Message::WindowOpened { hwnd })?;
        trace!("Window Created: {hwnd:?}");

        let mut msg = MSG::default();
        while msg.message != WM_QUIT {
            // Keep in mind that this blocks the windows thread until the next Windows message.
            if unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                match msg.message {
                    WM_KEYDOWN => {
                        message_sender.send(Message::KeyDown {  })?;
                    }
                    WM_KEYUP => {
                        message_sender.send(Message::KeyUp {  })?;
                    }
                    _ => {}
                }
            }
        }

        message_sender.send(Message::WindowClosed)?;
        trace!("Window Closed");

        message_sender.send(Message::Exit)?;
        Ok(())
    }

    pub fn set_visibility(&self, visibility: Visibility) {
        unsafe {
            ShowWindow(
                self.hwnd,
                match visibility {
                    Visibility::Shown => SW_SHOW,
                    Visibility::Hidden => SW_HIDE,
                },
            );
        }
    }
}

impl Iterator for AppWindow {
    type Item = Message;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(message) = self.message_receiver.try_recv() {
            if let Message::Exit = message {
                None
            } else {
                Some(message)
            }
        } else {
            Some(Message::Empty)
        }
    }
}
