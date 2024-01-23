use std::sync::mpsc::TryRecvError;

use messaging::{Mailbox, MessagingError};
use tracing::*;
use windows::Win32::{
  Foundation::*,
  UI::{Shell::DefSubclassProc, WindowsAndMessaging::*},
};

use super::message::{AppMessage, WindowMessage};

pub extern "system" fn wnd_proc(hwnd: HWND, message: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
  match message {
    WM_CLOSE => LRESULT(0),
    _ => unsafe { DefWindowProcW(hwnd, message, w_param, l_param) },
  }
}

pub extern "system" fn subclass_proc(
  hwnd: HWND,
  message: u32,
  w_param: WPARAM,
  l_param: LPARAM,
  _u_id_subclass: usize,
  dw_ref_data: usize,
) -> LRESULT {
  let mailbox = unsafe { &mut *(dw_ref_data as *mut Mailbox<WindowMessage, AppMessage>) };

  match mailbox.poll() {
    Ok(message) => match message {
      AppMessage::CloseRequested => {
        if let Err(error) = mailbox.send(WindowMessage::Closed) {
          error!("WindowMessage::Closed: {error}")
        }
      }
      AppMessage::Closed => {
        if let Err(error) = mailbox.send(WindowMessage::Exit) {
          error!("WindowMessage::Exit: {error}")
        };
        unsafe {
          if let Err(error) = DestroyWindow(hwnd) {
            error!("DestroyWindow(hwnd): {error}");
          }
        }
        return LRESULT(0);
      }
      _ => {}
    },
    Err(error) => {
      if let MessagingError::PollError {
        error: TryRecvError::Disconnected,
      } = error
      {
        error!("mailbox.poll: {error}")
      }
    }
  }

  let _ = mailbox
    .send(WindowMessage::new(hwnd, message, w_param, l_param))
    .inspect_err(|e| error!("WindowMessage::new: {e}"));

  match message {
    WM_CLOSE => LRESULT(0),
    _ => unsafe { DefSubclassProc(hwnd, message, w_param, l_param) },
  }
}
