// Reference for multithreaded input processing:
//   * https://www.jendrikillner.com/post/rust-game-part-3/
//   * https://github.com/jendrikillner/RustMatch3/blob/rust-game-part-3/
use std::{os::raw::c_void, sync::mpsc::channel};

use foxy_utils::{
  log::LogErr,
  types::{
    behavior::{CloseBehavior, ColorMode, Visibility},
    primitives::Dimensions,
    thread::EngineThread,
  },
};
use messaging::Mailbox;
use raw_window_handle::{
  HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};
use tracing::*;
use windows::{
  core::*,
  Win32::{
    Foundation::*,
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
    UI::WindowsAndMessaging::*,
  },
};

use self::window_loop::{WindowLoop, WindowLoopCreateInfo};
use crate::{
  debug::validation::ValidationLayer,
  window::{
    builder::{HasSize, HasTitle, MissingSize, MissingTitle, WindowBuilder, WindowCreateInfo},
    input::Input,
    message::{AppMessage, KeyboardMessage, MouseMessage, WindowMessage},
    state::WindowState,
  },
};

pub mod builder;
pub mod input;
pub mod message;
pub mod procs;
pub mod state;
mod window_loop;

#[allow(unused)]
pub struct Window {
  app_mailbox: Mailbox<AppMessage, WindowMessage>,
  state: WindowState,
  window_thread: EngineThread<WindowLoop>,
}

impl Drop for Window {
  fn drop(&mut self) {
    ValidationLayer::instance().shutdown();
  }
}

impl Window {
  pub const MSG_APP_MESSAGE: u32 = WM_USER + 11;
  pub const MSG_EXIT_LOOP: u32 = WM_USER + 69;
  pub const WINDOW_SUBCLASS_ID: usize = 0;
  pub const WINDOW_THREAD_ID: &'static str = "window";

  pub fn builder() -> WindowBuilder<MissingTitle, MissingSize> {
    Default::default()
  }

  pub fn new(create_info: WindowCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
    ValidationLayer::instance().init();

    let (mut app_mailbox, win32_mailbox) = Mailbox::new_entangled_pair();
    let (proc_sender, proc_receiver) = channel::<WindowMessage>();
    let mut window_thread = EngineThread::new(WindowLoop::new(win32_mailbox, proc_receiver));

    window_thread.run(WindowLoopCreateInfo::new(create_info.clone(), proc_sender));

    // block until first message sent (which will be the window opening)
    match app_mailbox.wait()? {
      WindowMessage::Ready { hwnd, hinstance } => {
        let input = Input::new();

        let mut window_rect = RECT::default();
        let _ = unsafe { GetWindowRect(hwnd, std::ptr::addr_of_mut!(window_rect)) }.log_error();
        let mut client_rect = RECT::default();
        let _ = unsafe { GetClientRect(hwnd, std::ptr::addr_of_mut!(client_rect)) }.log_error();

        let state = WindowState {
          hwnd,
          hinstance,
          size: Dimensions {
            width: window_rect.right - window_rect.left,
            height: window_rect.bottom - window_rect.top,
          },
          inner_size: Dimensions {
            width: client_rect.right - client_rect.left,
            height: client_rect.bottom - client_rect.top,
          },
          title: String::from(create_info.title.0),
          color_mode: create_info.color_mode,
          close_behavior: create_info.close_behavior,
          visibility: create_info.visibility,
          input,
        };

        let mut window = Self {
          app_mailbox,
          state,
          window_thread,
        };

        window.set_color_mode(window.state.color_mode);
        window.set_visibility(window.state.visibility);

        Ok(window)
      }
      _ => Err(anyhow::format_err!("Invalid message")),
    }
  }

  fn send_message_to_window(&self, message: AppMessage) -> anyhow::Result<()> {
    self.app_mailbox.send(message)?;
    unsafe {
      SendMessageW(self.state.hwnd, Self::MSG_APP_MESSAGE, WPARAM(0), LPARAM(0));
    }
    Ok(())
  }

  pub fn exit(&mut self) {
    if let Err(error) = self.send_message_to_window(AppMessage::DestroyWindow { hwnd: self.state.hwnd }) {
      error!("{error}");
    }
  }

  pub fn set_visibility(&mut self, visibility: Visibility) {
    self.state.visibility = visibility;
    unsafe {
      ShowWindow(self.state.hwnd, match visibility {
        Visibility::Shown => SW_SHOW,
        Visibility::Hidden => SW_HIDE,
      });
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

  pub fn size(&self) -> Dimensions {
    self.state.size
  }

  pub fn inner_size(&self) -> Dimensions {
    self.state.inner_size
  }

  /// Handles windows messages and then passes most onto the user
  fn intercept_message(&mut self, message: WindowMessage) -> Option<WindowMessage> {
    match message {
      WindowMessage::ExitLoop => {
        self.window_thread.join();
        return None;
      }
      WindowMessage::CloseRequested if CloseBehavior::Default == self.state.close_behavior => self.exit(),
      WindowMessage::Resized {
        window_rect,
        client_rect,
      } => {
        self.state.size = Dimensions {
          width: window_rect.right - window_rect.left,
          height: window_rect.bottom - window_rect.top,
        };
        self.state.inner_size = Dimensions {
          width: client_rect.right - client_rect.left,
          height: client_rect.bottom - client_rect.top,
        };
      }
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
      self.intercept_message(message)
    } else {
      None
    }
  }
}

impl Iterator for Window {
  type Item = WindowMessage;

  /// Returns next window message if available, otherwise returns an empty
  /// message immediately.
  ///
  /// Returns `None` when app is exiting.
  ///
  /// Use this if you want the application to run full tilt, as fast as
  /// possible.
  ///
  /// ***Note:** the window message thread will still block until a message is
  /// recieved from Windows.*
  fn next(&mut self) -> Option<Self::Item> {
    if let Ok(message) = self.app_mailbox.poll() {
      // info!("{message:?}");
      self.intercept_message(message)
    } else {
      Some(WindowMessage::None)
    }
  }
}

// impl HasWindowHandle for Window {
//   fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
//     Ok(unsafe { WindowHandle::borrow_raw(self.raw_window_handle()) })
//   }
// }

// impl HasDisplayHandle for Window {
//   fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
//     Ok(unsafe { DisplayHandle::borrow_raw(self.raw_display_handle()) })
//   }
// }

unsafe impl HasRawWindowHandle for Window {
  fn raw_window_handle(&self) -> RawWindowHandle {
    let mut handle = Win32WindowHandle::empty();
    handle.hwnd = self.state.hwnd.0 as *mut c_void;
    handle.hinstance = self.state.hinstance.0 as *mut c_void;
    // let mut handle =
    //   Win32WindowHandle::new(NonZeroIsize::new(self.state.hwnd.0).expect("window
    // handle should not be zero")); let hinstance =
    // NonZeroIsize::new(self.state.hinstance.0).expect("instance handle should not
    // be zero"); handle.hinstance = Some(hinstance);
    RawWindowHandle::Win32(handle)
  }
}

unsafe impl HasRawDisplayHandle for Window {
  fn raw_display_handle(&self) -> RawDisplayHandle {
    RawDisplayHandle::Windows(WindowsDisplayHandle::empty())
  }
}
