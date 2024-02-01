use std::thread::JoinHandle;

use crossbeam::channel::{Receiver, Sender, TryRecvError};
use foxy_utils::{
  log::LogErr,
  mailbox::{Mailbox, MessagingError},
  thread::{error::ThreadError, handle::ThreadLoop},
};
use tracing::*;
use windows::{
  core::{HSTRING, PCWSTR},
  Win32::{
    Foundation::HINSTANCE,
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
  main_message::MainMessage,
  window_message::WindowMessage,
};
use crate::window::Window;

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

pub struct WindowLoop {
  mailbox: Mailbox<WindowMessage, MainMessage>,
  proc_receiver: Receiver<WindowMessage>,
}

impl ThreadLoop for WindowLoop {
  type Params = WindowThreadCreateInfo;

  fn run(mut self, thread_id: String, info: Self::Params) -> Result<JoinHandle<Result<(), ThreadError>>, ThreadError> {
    std::thread::Builder::new()
      .name(thread_id)
      .spawn(move || -> Result<(), ThreadError> {
        let hinstance: HINSTANCE = unsafe { GetModuleHandleW(None).map_err(anyhow::Error::from)? }.into();
        debug_assert_ne!(hinstance.0, 0);
        let htitle = HSTRING::from(info.create_info.title.0);
        let window_class = htitle.clone();

        let wc = WNDCLASSEXW {
          cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
          style: CS_VREDRAW | CS_HREDRAW | CS_DBLCLKS,
          cbWndExtra: std::mem::size_of::<WNDCLASSEXW>() as i32,
          lpfnWndProc: Some(crate::window::procs::wnd_proc),
          hInstance: hinstance,
          hCursor: unsafe { LoadCursorW(None, IDC_ARROW).map_err(anyhow::Error::from)? },
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
            info.create_info.size.width,
            info.create_info.size.height,
            None,
            None,
            hinstance,
            None,
          )
        };

        let window_data_ptr = Box::into_raw(Box::new(info.proc_sender));

        unsafe {
          windows::Win32::UI::Shell::SetWindowSubclass(
            hwnd,
            Some(crate::window::procs::subclass_proc),
            Window::WINDOW_SUBCLASS_ID,
            window_data_ptr as usize,
          );
        }

        // Send opened message to main function
        self
          .mailbox
          .send(WindowMessage::Ready { hwnd, hinstance })
          .map_err(anyhow::Error::from)?;

        loop {
          match self.proc_receiver.try_recv() {
            Ok(message) => {
              let _ = self.mailbox.send(message).log_error_msg("WindowMessage::new");
              match self.mailbox.try_recv() {
                Ok(MainMessage::Close) => {
                  let _ = self.mailbox.send(WindowMessage::Closing).log_error();
                  let _ = unsafe { DestroyWindow(hwnd) }.log_error();
                }
                Err(MessagingError::TryRecvError {
                  error: TryRecvError::Disconnected,
                }) => {
                  error!("channel between main and window was closed!");
                }
                _ => {}
              }
            }
            Err(TryRecvError::Disconnected) => {
              error!("channel between window and window proc was closed!");
              break;
            }
            Err(TryRecvError::Empty) => {
              if self.next_message().is_none() {
                break;
              }
            }
          }
        }

        // Send exit message to main function
        self
          .mailbox
          .send(WindowMessage::ExitLoop)
          .map_err(anyhow::Error::from)?;

        Ok(())
      })
      .map_err(ThreadError::from)
  }
}

impl WindowLoop {
  pub fn new(mailbox: Mailbox<WindowMessage, MainMessage>, proc_receiver: Receiver<WindowMessage>) -> Self {
    Self { mailbox, proc_receiver }
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
