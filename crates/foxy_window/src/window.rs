// Reference for multithreaded input processing:
//   * https://www.jendrikillner.com/post/rust-game-part-3/
//   * https://github.com/jendrikillner/RustMatch3/blob/rust-game-part-3/
use std::os::raw::c_void;

use crossbeam::channel::{Receiver, TryRecvError};
use foxy_utils::{
  log::LogErr,
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
  stage::Stage,
  window_loop::{WindowLoop, WindowThreadCreateInfo},
  window_message::StateMessage,
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
pub mod stage;
pub mod state;
pub mod window_loop;
pub mod window_message;

#[allow(unused)]
pub struct Window {
  state: WindowState,
  window_thread: LoopHandle<WindowLoop, WindowThreadCreateInfo>,
  proc_receiver: Receiver<WindowMessage>,
  // input_queue: PriorityQueue<WindowMessage, MessagePriority>,
  current_stage: Stage,
}

impl Drop for Window {
  fn drop(&mut self) {
    ValidationLayer::instance().shutdown();
  }
}

impl Window {
  pub const MSG_EXIT_LOOP: u32 = WM_USER + 69;
  pub const MSG_MAIN_CLOSE_REQ: u32 = WM_USER + 11;
  pub const WINDOW_SUBCLASS_ID: usize = 0;
  pub const WINDOW_THREAD_ID: &'static str = "window";

  pub fn builder() -> WindowBuilder<MissingTitle, MissingSize> {
    Default::default()
  }

  pub fn new(create_info: WindowCreateInfo<HasTitle, HasSize>) -> Result<Self, WindowError> {
    ValidationLayer::instance().init();
    
    let (proc_sender, proc_receiver) = crossbeam::channel::unbounded();

    let winloop = WindowLoop::new();
    let wincreate_info = WindowThreadCreateInfo::new(create_info.clone(), proc_sender);
    let mut window_thread = LoopHandle::new(vec![Self::WINDOW_THREAD_ID.into()], winloop, wincreate_info);

    window_thread.run();

    // block until first message sent (which will be the window opening)
    if let WindowMessage::State(StateMessage::Ready { hwnd, hinstance }) =
      proc_receiver.recv().map_err(anyhow::Error::from)?
    {
      let input = Input::new();

      let mut window_rect = RECT::default();
      let _ = unsafe { GetWindowRect(HWND(hwnd), std::ptr::addr_of_mut!(window_rect)) }.log_error();
      let mut client_rect = RECT::default();
      let _ = unsafe { GetClientRect(HWND(hwnd), std::ptr::addr_of_mut!(client_rect)) }.log_error();

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
        state,
        window_thread,
        proc_receiver,
        // input_queue: Default::default(),
        current_stage: Stage::Looping,
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
      ShowWindow(HWND(self.state.hwnd), match visibility {
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
        HWND(self.state.hwnd),
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
      let _ = SetWindowTextW(HWND(self.state.hwnd), &HSTRING::from(title)).log_error();
    }
  }

  pub fn size(&self) -> Dimensions {
    self.state.size
  }

  pub fn inner_size(&self) -> Dimensions {
    self.state.inner_size
  }

  pub fn close(&mut self) {
    self.current_stage = Stage::Exiting;
    // let _ = unsafe { PostMessageW(HWND(self.state.hwnd),
    // Self::MSG_MAIN_CLOSE_REQ, WPARAM(0), LPARAM(0)) }.log_error();
  }

  fn handle_message(&mut self, message: WindowMessage) -> Option<WindowMessage> {
    // match &message {
    //   WindowMessage::Other { .. } => {}
    //   _ => debug!("{message:?}"),
    // }

    // self.enqueue_message(message);

    // let (message, _priority) = self.input_queue.pop()?;

    match message {
      WindowMessage::CloseRequested => {
        // TODO: Add manual custom close behavior back
        debug!("Close Requested");
        self.close();
      }
      // WindowMessage::CloseRequested => {
      //   let _ =
      //     unsafe { PostMessageW(HWND(self.state.hwnd), Self::MSG_MAIN_CLOSE_REQ, WPARAM(0), LPARAM(0)) }.log_error();
      // }
      // WindowMessage::Closing => {
      //   debug!("Closing");
      //   let _ =
      //     unsafe { PostMessageW(HWND(self.state.hwnd), Self::MSG_MAIN_CLOSING, WPARAM(0), LPARAM(0)) }.log_error();
      // }
      // WindowMessage::ExitLoop => {
      //   // TODO: Should happen only with default behavior
      //   debug!("ExitLoop");
      //   return None;
      // }
      WindowMessage::State(StateMessage::Resized {
        window_size,
        client_size,
      }) => {
        self.state.size = window_size;
        self.state.inner_size = client_size;
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

  // fn enqueue_message(&mut self, message: WindowMessage) {
  //   let priority = match message {
  //     WindowMessage::CloseRequested => MessagePriority::High,
  //     WindowMessage::Keyboard(KeyboardMessage::Key { .. }) => MessagePriority::High,
  //     WindowMessage::Mouse(MouseMessage::Button { .. }) => MessagePriority::High,
  //     _ => MessagePriority::Low,
  //   };

  //   self.input_queue.push(message, priority);
  // }

  fn next_message<const SHOULD_WAIT: bool>(&mut self) -> Option<WindowMessage> {
    match self.current_stage {
      Stage::Looping => {
        if SHOULD_WAIT {
          match self.proc_receiver.recv() {
            Ok(message) => self.handle_message(message),
            _ => {
              error!("channel between main and window was closed!");
              self.current_stage = Stage::Exiting;
              Some(WindowMessage::None)
            }
          }
        } else {
          match self.proc_receiver.try_recv() {
            Ok(message) => self.handle_message(message),
            Err(TryRecvError::Disconnected) => {
              error!("channel between main and window was closed!");
              self.current_stage = Stage::Exiting;
              Some(WindowMessage::None)
            }
            _ => Some(WindowMessage::None),
          }
        }
      }
      Stage::Exiting => {
        self.current_stage = Stage::ExitLoop;
        Some(WindowMessage::Closing)
      }
      Stage::ExitLoop => {
        let _ =
          unsafe { PostMessageW(HWND(self.state.hwnd), Self::MSG_MAIN_CLOSE_REQ, WPARAM(0), LPARAM(0)) }.log_error();
        self.window_thread.join();
        None
      }
    }
  }

  /// Waits for next window message before returning.
  ///
  /// Returns `None` when app is exiting.
  ///
  /// Use this if you want the application to only react to window events.
  #[allow(unused)]
  pub fn wait(&mut self) -> Option<WindowMessage> {
    self.next_message::<true>()
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
    self.next_message::<false>()
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
    handle.hwnd = self.state.hwnd as *mut c_void;
    handle.hinstance = self.state.hinstance as *mut c_void;
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
