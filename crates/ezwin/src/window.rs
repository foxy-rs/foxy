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

use crate::window::builder::ColorMode;

use self::{
    builder::{CloseBehavior, HasSize, HasTitle, Visibility, WindowCreateInfo},
    input::Input,
    message::{AppMessage, KeyboardMessage, MouseMessage, WindowMessage},
    state::WindowState,
};

pub mod builder;
pub mod input;
pub mod message;
pub mod procs;
pub mod state;

struct WindowChannels {
    pub window_message_sender: Sender<WindowMessage>,
    pub app_message_receiver: Receiver<AppMessage>,
}

#[derive(Debug)]
#[allow(unused)]
pub struct Window {
    window_message_receiver: Receiver<WindowMessage>,
    app_message_sender: Sender<AppMessage>,
    state: WindowState,
}

impl Window {
    pub const WINDOW_THREAD_ID: &'static str = "window";
    pub const WINDOW_STATE_PTR_ID: usize = 0;
    pub const WINDOW_SUBCLASS_ID: usize = 0;

    pub fn new(create_info: WindowCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
        let htitle = HSTRING::from(create_info.title.0);

        let (window_message_sender, window_message_receiver) = std::sync::mpsc::channel();
        let (app_message_sender, app_message_receiver) = std::sync::mpsc::channel();

        std::thread::Builder::new()
            .name(Self::WINDOW_THREAD_ID.into())
            .spawn(move || -> anyhow::Result<()> {
                let instance = unsafe { GetModuleHandleW(None)? };
                debug_assert_ne!(instance.0, 0);
                let window_class = htitle.clone();

                let wc = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    style: CS_VREDRAW | CS_HREDRAW | CS_DBLCLKS,
                    cbWndExtra: std::mem::size_of::<WNDCLASSEXW>() as i32,
                    lpfnWndProc: Some(crate::window::procs::wnd_proc),
                    hInstance: instance.into(),
                    hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
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
                        &htitle,
                        WS_OVERLAPPEDWINDOW,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        create_info.size.width,
                        create_info.size.height,
                        None,
                        None,
                        instance,
                        None,
                    )
                };

                // Send opened message before surrendering the message sender
                window_message_sender.send(WindowMessage::Ready { hwnd })?;

                let window_channels = WindowChannels {
                    window_message_sender,
                    app_message_receiver,
                };

                let window_channels_ptr = Box::into_raw(Box::new(window_channels));

                unsafe {
                    windows::Win32::UI::Shell::SetWindowSubclass(
                        hwnd,
                        Some(crate::window::procs::subclass_proc),
                        Self::WINDOW_SUBCLASS_ID,
                        window_channels_ptr as usize,
                    );
                }

                while let Some(_message) = Self::next_message() {}

                Ok(())
            })?;

        // block until first message sent (which will be the window opening)
        match window_message_receiver.recv()? {
            WindowMessage::Ready { hwnd } => {
                let input = Input::new();
                let state = WindowState {
                    hwnd,
                    width: create_info.size.width,
                    height: create_info.size.height,
                    title: String::from(create_info.title.0),
                    color_mode: create_info.color_mode,
                    close_behavior: create_info.close_behavior,
                    visibility: create_info.visibility,
                    input,
                };

                let mut window = Self {
                    window_message_receiver,
                    app_message_sender,
                    state,
                };

                window.set_color_mode(window.state.color_mode);
                window.set_visibility(window.state.visibility);

                Ok(window)
            }
            _ => Err(anyhow::format_err!("Invalid message")),
        }
    }

    fn next_message() -> Option<WindowMessage> {
        let mut msg = MSG::default();
        // Keep in mind that this blocks the windows thread until the next Windows message.
        if unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            Some(WindowMessage::new(
                msg.hwnd,
                msg.message,
                msg.wParam,
                msg.lParam,
            ))
        } else {
            None
        }
    }

    pub fn close(&self) {
        if let Err(error) = self.app_message_sender.send(AppMessage::Exit) {
            error!("{error}");
        }
    }

    pub fn set_visibility(&mut self, visibility: Visibility) {
        self.state.visibility = visibility;
        unsafe {
            ShowWindow(
                self.state.hwnd,
                match visibility {
                    Visibility::Shown => SW_SHOW,
                    Visibility::Hidden => SW_HIDE,
                },
            );
        }
    }

    pub fn set_color_mode(&mut self, color_mode: ColorMode) {
        self.state.color_mode = color_mode;
        let dark_mode = BOOL::from(color_mode == ColorMode::Dark);
        if let Err(error) = unsafe {
            DwmSetWindowAttribute(
                self.state.hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                std::ptr::addr_of!(dark_mode) as *const std::ffi::c_void,
                std::mem::size_of::<BOOL>() as u32,
            )
        } {
            error!("{error}");
        };
    }

    pub fn title(&self) -> &str {
        &self.state.title
    }

    pub fn set_title(&self, title: &str) {
        unsafe {
            let _ = SetWindowTextW(self.state.hwnd, &HSTRING::from(title));
        }
    }

    #[allow(unused)]
    pub fn raw_window_handle(&self) -> HWND {
        self.state.hwnd
    }

    /// Handles windows messages and then passes most onto the user
    fn message_handler(&mut self, message: WindowMessage) -> Option<WindowMessage> {
        match message {
            WindowMessage::Exit => return None,
            WindowMessage::CloseRequested
                if CloseBehavior::Default == self.state.close_behavior =>
            {
                self.close()
            }
            WindowMessage::Keyboard(KeyboardMessage::Key {
                key_code, state, ..
            }) => {
                self.state.input.update_keyboard_state(key_code, state);
            }
            WindowMessage::Mouse(MouseMessage::Button {
                mouse_code, state, ..
            }) => {
                self.state
                    .input
                    .update_mouse_button_state(mouse_code, state);
            }
            _ => {}
        }

        Some(message)
    }

    /// Waits for next window message before returning.
    ///
    /// Returns `None` when app is exiting.
    ///
    /// Use this if you want the application to only react to window events.
    #[allow(unused)]
    pub fn wait(&mut self) -> Option<WindowMessage> {
        if let Ok(message) = self.window_message_receiver.recv() {
            self.message_handler(message)
        } else {
            None
        }
    }
}

impl Iterator for Window {
    type Item = WindowMessage;

    /// Returns next window message if available, otherwise returns an empty message immediately.
    ///
    /// Returns `None` when app is exiting.
    ///
    /// Use this if you want the application to run full tilt, as fast as possible.
    ///
    /// ***Note:** the window message thread will still block until a message is recieved from Windows.*
    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(message) = self.window_message_receiver.try_recv() {
            self.message_handler(message)
        } else {
            Some(WindowMessage::Empty)
        }
    }
}
