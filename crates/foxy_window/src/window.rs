// Reference for multithreaded input processing:
//   * https://www.jendrikillner.com/post/rust-game-part-3/
//   * https://github.com/jendrikillner/RustMatch3/blob/rust-game-part-3/
use std::os::raw::c_void;

use crossbeam::channel::TryRecvError;
use foxy_utils::{
  log::LogErr,
  mailbox::{Mailbox, MessagingError},
  thread::handle::LoopHandle,
  types::{
    behavior::{ColorMode, Visibility},
    primitives::Dimensions,
  },
};
use raw_window_handle::{
  HasRawDisplayHandle,
  HasRawWindowHandle,
  RawDisplayHandle,
  RawWindowHandle,
  Win32WindowHandle,
  WindowsDisplayHandle,
};
use tracing::*;
use windows::{
  core::HSTRING,
  Win32::{
    Foundation::*,
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
    UI::WindowsAndMessaging::*,
  },
};

use self::{
  main_message::MainMessage,
  window_loop::{WindowLoop, WindowThreadCreateInfo},
};
use crate::{
  debug::{error::WindowError, validation::ValidationLayer},
  window::{
    builder::{HasSize, HasTitle, MissingSize, MissingTitle, WindowBuilder, WindowCreateInfo},
    input::Input,
    state::WindowState,
    window_message::{KeyboardMessage, MouseMessage, WindowMessage},
  },
};

pub mod builder;
pub mod input;
pub mod main_message;
pub mod procs;
pub mod state;
pub mod window_loop;
pub mod window_message;

#[allow(unused)]
pub struct Window {
  mailbox: Mailbox<MainMessage, WindowMessage>,
  state: WindowState,
  window_thread: LoopHandle<WindowLoop, WindowThreadCreateInfo>,
}

impl Drop for Window {
  fn drop(&mut self) {
    ValidationLayer::instance().shutdown();
  }
}

impl Window {
  pub const MSG_EXIT_LOOP: u32 = WM_USER + 69;
  pub const MSG_MESSAGE_FROM_MAIN: u32 = WM_USER + 11;
  pub const WINDOW_SUBCLASS_ID: usize = 0;
  pub const WINDOW_THREAD_ID: &'static str = "window";

  pub fn builder() -> WindowBuilder<MissingTitle, MissingSize> {
    Default::default()
  }

  pub fn new(create_info: WindowCreateInfo<HasTitle, HasSize>) -> Result<Self, WindowError> {
    ValidationLayer::instance().init();

    let (main_mailbox, window_mailbox) = Mailbox::new_entangled_pair();
    let (proc_sender, proc_receiver) = crossbeam::channel::unbounded();

    let winloop = WindowLoop::new(window_mailbox, proc_receiver);
    let wincreate_info = WindowThreadCreateInfo::new(create_info.clone(), proc_sender);
    let mut window_thread = LoopHandle::new(Self::WINDOW_THREAD_ID, winloop, wincreate_info);

    window_thread.run();

    // block until first message sent (which will be the window opening)
    if let WindowMessage::Ready { hwnd, hinstance } = main_mailbox.recv().map_err(anyhow::Error::from)? {
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
        visibility: create_info.visibility,
        input,
      };

      let mut window = Self {
        mailbox: main_mailbox,
        state,
        window_thread,
      };

      window.set_color_mode(window.state.color_mode);
      window.set_visibility(window.state.visibility);

      Ok(window)
    } else {
      Err(anyhow::format_err!("Invalid message").into())
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
      let _ = SetWindowTextW(self.state.hwnd, &HSTRING::from(title)).log_error();
    }
  }

  pub fn size(&self) -> Dimensions {
    self.state.size
  }

  pub fn inner_size(&self) -> Dimensions {
    self.state.inner_size
  }

  pub fn close(&mut self) {
    let _ = self.send_message_to_window(MainMessage::Close).log_error();
    loop {
      // consume all messages until closed or error encountered
      match self.mailbox.recv().log_error() {
        Ok(WindowMessage::Closing) => break,
        Ok(_) => {}
        Err(_) => {
          error!("never received Closing message! breaking!");
          break;
        }
      }
    }
    self.window_thread.join();

    // match self.mailbox.recv() {
    //   Ok(WindowMessage::Closing) => {
    //     // next message MUST be Closing
    //     let _ = self.send_message_to_window(MainMessage::Exit).log_error();
    //   }
    //   Ok(_) => panic!("expected window closing message!"),
    //   Err(error) => error!("{error}"),
    // }
  }

  fn send_message_to_window(&self, message: MainMessage) -> Result<(), WindowError> {
    self.mailbox.send(message).map_err(anyhow::Error::from)?;
    unsafe {
      // This isn't sending data, just prompting the wndproc to wake up and process
      // the message in the mailbox
      SendMessageW(self.state.hwnd, Self::MSG_MESSAGE_FROM_MAIN, WPARAM(0), LPARAM(0));
    }
    Ok(())
  }

  /// Handles windows messages and then passes most onto the user
  fn intercept_message(&mut self, message: WindowMessage) -> Option<WindowMessage> {
    match message {
      WindowMessage::ExitLoop => {
        self.window_thread.join();
        return None;
      }
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
    if let Ok(message) = self.mailbox.recv() {
      self.intercept_message(message)
    } else {
      error!("channel between main and window was closed!");
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
    match self.mailbox.try_recv() {
      Ok(message) => self.intercept_message(message),
      Err(MessagingError::TryRecvError {
        error: TryRecvError::Disconnected,
      }) => {
        error!("channel between main and window was closed!");
        None
      }
      _ => Some(WindowMessage::None),
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
