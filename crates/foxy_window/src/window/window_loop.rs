use std::{
  sync::mpsc::{Receiver, Sender, TryRecvError},
  thread::JoinHandle,
};

use foxy_types::thread::ThreadLoop;
use foxy_util::log::LogErr;
use messaging::{Mailbox, MessagingError};
use tracing::*;
use windows::{
  core::{HSTRING, PCWSTR},
  Win32::{
    Foundation::HINSTANCE,
    System::LibraryLoader::GetModuleHandleW,
    UI::WindowsAndMessaging::{
      CreateWindowExW, DestroyWindow, DispatchMessageW, GetMessageW, LoadCursorW, RegisterClassExW, TranslateMessage,
      CS_DBLCLKS, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, MSG, WINDOW_EX_STYLE, WNDCLASSEXW,
      WS_OVERLAPPEDWINDOW,
    },
  },
};

use crate::window::Window;

use super::{
  builder::{HasSize, HasTitle, WindowCreateInfo},
  message::{AppMessage, WindowMessage},
};

pub struct WindowLoopCreateInfo {
  create_info: WindowCreateInfo<HasTitle, HasSize>,
  proc_sender: Sender<WindowMessage>,
}

impl WindowLoopCreateInfo {
  pub fn new(create_info: WindowCreateInfo<HasTitle, HasSize>, proc_sender: Sender<WindowMessage>) -> Self {
    Self {
      create_info,
      proc_sender,
    }
  }
}

pub struct WindowLoop {
  mailbox: Mailbox<WindowMessage, AppMessage>,
  proc_receiver: Receiver<WindowMessage>,
}

impl ThreadLoop for WindowLoop {
  const THREAD_ID: &'static str = "window";
  type Params = WindowLoopCreateInfo;

  fn run(mut self, info: Self::Params) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    std::thread::Builder::new()
      .name(Self::THREAD_ID.into())
      .spawn(move || -> anyhow::Result<()> {
        let hinstance: HINSTANCE = unsafe { GetModuleHandleW(None)? }.into();
        debug_assert_ne!(hinstance.0, 0);
        let htitle = HSTRING::from(info.create_info.title.0);
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
        self.mailbox.send(WindowMessage::Ready { hwnd, hinstance })?;

        loop {
          match self.proc_receiver.try_recv() {
            Ok(message) => {
              let _ = self.mailbox.send(message).log_error_msg("WindowMessage::new");

              match self.mailbox.poll() {
                Ok(AppMessage::DestroyWindow { hwnd }) => {
                  let _ = unsafe { DestroyWindow(hwnd) }.log_error();
                }
                Err(MessagingError::PollError {
                  error: TryRecvError::Disconnected,
                }) => {
                  error!("window loop mailbox disconnected")
                }
                _ => {}
              }
            }
            Err(TryRecvError::Disconnected) => {
              error!("window proc channel disconnected")
            }
            Err(TryRecvError::Empty) => {
              if self.next_message().is_none() {
                break;
              }
            }
          }
        }

        // Send exit message to main function
        self.mailbox.send(WindowMessage::ExitLoop)?;

        Ok(())
      })
      .map_err(anyhow::Error::from)
  }
}

impl WindowLoop {
  pub fn new(mailbox: Mailbox<WindowMessage, AppMessage>, proc_receiver: Receiver<WindowMessage>) -> Self {
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
