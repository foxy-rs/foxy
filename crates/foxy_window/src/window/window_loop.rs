use std::sync::{Arc, RwLock};

use crossbeam::channel::Sender;
use foxy_utils::{
  log::LogErr,
  thread::{
    error::ThreadError,
    handle::{HandlesResult, ThreadLoop},
  }, thread_error,
};
use windows::{
  core::{HSTRING, PCWSTR},
  Win32::{
    Foundation::{HINSTANCE, HWND},
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
      CreateWindowExW,
      DestroyWindow,
      DispatchMessageW,
      GetMessageW,
      LoadCursorW,
      RegisterClassExW,
      TranslateMessage,
      CS_DBLCLKS,
      CS_HREDRAW,
      CS_VREDRAW,
      CW_USEDEFAULT,
      IDC_ARROW,
      MSG,
      WINDOW_EX_STYLE,
      WNDCLASSEXW,
      WS_OVERLAPPEDWINDOW,
    },
  },
};

use super::{
  builder::{HasSize, HasTitle, WindowCreateInfo},
  window_message::WindowMessage,
};
use crate::window::{procs::SubclassWindowData, window_message::StateMessage, Window};

pub struct WindowThreadCreateInfo {
  create_info: WindowCreateInfo<HasTitle, HasSize>,
  proc_sender: Sender<WindowMessage>,
}

impl WindowThreadCreateInfo {
  pub fn new(create_info: WindowCreateInfo<HasTitle, HasSize>, proc_sender: Sender<WindowMessage>) -> Self {
    Self {
      create_info,
      proc_sender,
    }
  }
}

pub struct WindowLoop;

impl ThreadLoop for WindowLoop {
  type Params = WindowThreadCreateInfo;

  fn run(mut self, thread_id: String, info: Self::Params) -> HandlesResult {

    let hwnd = Arc::new(RwLock::new(HWND::default()));

    // WINDOW
      std::thread::Builder::new()
        .name(thread_id)
        .spawn(move || -> Result<(), ThreadError> {
          let hinstance: HINSTANCE = unsafe { GetModuleHandleW(None).map_err(|e| thread_error!(e))? }.into();
          debug_assert_ne!(hinstance.0, 0);
          let htitle = HSTRING::from(info.create_info.title.0);
          let window_class = htitle.clone();

          let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_VREDRAW | CS_HREDRAW | CS_DBLCLKS,
            cbWndExtra: std::mem::size_of::<WNDCLASSEXW>() as i32,
            lpfnWndProc: Some(crate::window::procs::wnd_proc),
            hInstance: hinstance,
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW).map_err(|e| thread_error!(e))? },
            lpszClassName: PCWSTR(window_class.as_ptr()),
            ..Default::default()
          };

          unsafe {
            let atom = RegisterClassExW(&wc);
            debug_assert_ne!(atom, 0);
          }

          *hwnd.write().unwrap() = unsafe {
            CreateWindowExW(
              WINDOW_EX_STYLE::default(),
              &window_class,
              &htitle,
              WS_OVERLAPPEDWINDOW,
              CW_USEDEFAULT,
              CW_USEDEFAULT,
              info.create_info.size.width,
              info.create_info.size.height,
              None,
              None,
              hinstance,
              None,
            )
          };

          let window_data_ptr = Box::into_raw(Box::new(SubclassWindowData {
            sender: info.proc_sender.clone(),
          }));

          unsafe {
            windows::Win32::UI::Shell::SetWindowSubclass(
              *hwnd.read().unwrap(),
              Some(crate::window::procs::subclass_proc),
              Window::WINDOW_SUBCLASS_ID,
              window_data_ptr as usize,
            );
          }

          // Send opened message to main function
          info
            .proc_sender
            .send(WindowMessage::State(StateMessage::Ready {
              hwnd: hwnd.read().unwrap().0,
              hinstance: hinstance.0,
            }))
            .map_err(|e| thread_error!(e))?;

          // Message pump
          while let Some(message) = self.next_message() {
            if let WindowMessage::Other {
              message: Window::MSG_MAIN_CLOSE_REQ,
              ..
            } = message
            {
              let _ = unsafe { DestroyWindow(*hwnd.read().unwrap()) }.log_error();
              break;
            }
          }

          Ok(())
        })
        .map_err(ThreadError::from)
  }
}

impl WindowLoop {
  pub fn new() -> Self {
    Self
  }

  fn next_message(&mut self) -> Option<WindowMessage> {
    let mut msg = MSG::default();
    if unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } {
      unsafe {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
      }
      Some(WindowMessage::new(msg.hwnd, msg.message, msg.wParam, msg.lParam))
    } else {
      None
    }
  }
}

impl Default for WindowLoop {
  fn default() -> Self {
    Self::new()
  }
}
