// Reference for multithreaded input processing:
//   * https://www.jendrikillner.com/post/rust-game-part-3/
//   * https://github.com/jendrikillner/RustMatch3/blob/rust-game-part-3/

use std::{num::NonZeroIsize, prelude::v1::Result};

use messaging::Mailbox;
use raw_window_handle::{
  DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, Win32WindowHandle,
  WindowHandle, WindowsDisplayHandle,
};
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

use crate::{prelude::ValidationLayer, window::builder::ColorMode};

use self::{
  builder::{CloseBehavior, HasSize, HasTitle, Visibility, WindowCreateInfo},
  input::Input,
  message::{AppMessage, KeyboardMessage, MouseMessage, WindowMessage},
  state::{WindowSize, WindowState},
};

pub mod builder;
pub mod input;
pub mod message;
pub mod procs;
pub mod state;

// struct AppMessenger {
//     pub sender: Sender<AppMessage>,
//     pub receiver: Receiver<WindowMessage>,
// }

#[derive(Debug)]
#[allow(unused)]
pub struct Window {
  // window_message_receiver: Receiver<WindowMessage>,
  // app_message_sender: Sender<AppMessage>,
  app_mailbox: Mailbox<AppMessage, WindowMessage>,
  state: WindowState,
}

impl Drop for Window {
  fn drop(&mut self) {
    ValidationLayer::instance().shutdown();
  }
}

impl Window {
  pub const WINDOW_THREAD_ID: &'static str = "window";
  pub const WINDOW_STATE_PTR_ID: usize = 0;
  pub const WINDOW_SUBCLASS_ID: usize = 0;

  pub fn new(create_info: WindowCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
    ValidationLayer::instance().init();

    let (mut app_mailbox, win32_mailbox) = Mailbox::new_entangled_pair();
    Self::spawn_window_thread(create_info.clone(), win32_mailbox)?;

    // block until first message sent (which will be the window opening)
    match app_mailbox.wait()? {
      WindowMessage::Ready { hwnd, hinstance } => {
        let input = Input::new();
        let state = WindowState {
          hwnd,
          hinstance,
          size: WindowSize {
            width: create_info.size.width,
            height: create_info.size.height,
          },
          title: String::from(create_info.title.0),
          color_mode: create_info.color_mode,
          close_behavior: create_info.close_behavior,
          visibility: create_info.visibility,
          input,
        };

        let mut window = Self { app_mailbox, state };

        window.set_color_mode(window.state.color_mode);
        window.set_visibility(window.state.visibility);

        Ok(window)
      }
      _ => Err(anyhow::format_err!("Invalid message")),
    }
  }

  fn spawn_window_thread(
    create_info: WindowCreateInfo<HasTitle, HasSize>,
    win32_mailbox: Mailbox<WindowMessage, AppMessage>,
  ) -> anyhow::Result<()> {
    let htitle = HSTRING::from(create_info.title.0);
    std::thread::Builder::new()
      .name(Self::WINDOW_THREAD_ID.into())
      .spawn(move || -> anyhow::Result<()> {
        let hinstance: HINSTANCE = unsafe { GetModuleHandleW(None)? }.into();
        debug_assert_ne!(hinstance.0, 0);
        let window_class = htitle.clone();

        let wc = WNDCLASSEXW {
          cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
          style: CS_VREDRAW | CS_HREDRAW | CS_DBLCLKS,
          cbWndExtra: std::mem::size_of::<WNDCLASSEXW>() as i32,
          lpfnWndProc: Some(crate::window::procs::wnd_proc),
          hInstance: hinstance,
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
            hinstance,
            None,
          )
        };

        // Send opened message before surrendering the message sender
        win32_mailbox.send(WindowMessage::Ready { hwnd, hinstance })?;

        let window_channels_ptr = Box::into_raw(Box::new(win32_mailbox));

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

    Ok(())
  }

  fn next_message() -> Option<WindowMessage> {
    let mut msg = MSG::default();
    // Keep in mind that this blocks the windows thread until the next Windows message.
    if unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
      unsafe {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      }
      Some(WindowMessage::new(msg.hwnd, msg.message, msg.wParam, msg.lParam))
    } else {
      None
    }
  }

  pub fn close(&self) {
    if let Err(error) = self.app_mailbox.send(AppMessage::RequestExit) {
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
      if let Err(error) = SetWindowTextW(self.state.hwnd, &HSTRING::from(title)) {
        error!("{error}");
      }
    }
  }

  pub fn size(&self) -> WindowSize {
    self.state.size
  }

  #[allow(unused)]
  pub fn raw_window_handle(&self) -> RawWindowHandle {
    let mut handle =
      Win32WindowHandle::new(NonZeroIsize::new(self.state.hwnd.0).expect("window handle should not be zero"));
    let hinstance = NonZeroIsize::new(self.state.hinstance.0).expect("instance handle should not be zero");
    handle.hinstance = Some(hinstance);
    RawWindowHandle::Win32(handle)
  }

  #[allow(unused)]
  pub fn raw_display_handle(&self) -> RawDisplayHandle {
    RawDisplayHandle::Windows(WindowsDisplayHandle::new())
  }

  /// Handles windows messages and then passes most onto the user
  fn message_handler(&mut self, message: WindowMessage) -> Option<WindowMessage> {
    match message {
      WindowMessage::Exit => return None,
      WindowMessage::CloseRequested if CloseBehavior::Default == self.state.close_behavior => self.close(),
      WindowMessage::Keyboard(KeyboardMessage::Key { key_code, state, .. }) => {
        self.state.input.update_keyboard_state(key_code, state);
      }
      WindowMessage::Mouse(MouseMessage::Button { mouse_code, state, .. }) => {
        self.state.input.update_mouse_button_state(mouse_code, state);
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
    if let Ok(message) = self.app_mailbox.wait() {
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
    if let Ok(message) = self.app_mailbox.poll() {
      self.message_handler(message)
    } else {
      Some(WindowMessage::Empty)
    }
  }
}

impl HasWindowHandle for Window {
  fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
    Ok(unsafe { WindowHandle::borrow_raw(self.raw_window_handle()) })
  }
}

impl HasDisplayHandle for Window {
  fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
    Ok(unsafe { DisplayHandle::borrow_raw(self.raw_display_handle()) })
  }
}
