use messaging::Mailbox;
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

  if let Ok(AppMessage::RequestExit) = mailbox.poll() {
    if let Err(error) = mailbox.send(WindowMessage::Closed) {
      error!("{error}")
    };
    if let Err(error) = mailbox.send(WindowMessage::Exit) {
      error!("{error}")
    };
    unsafe {
      if let Err(error) = DestroyWindow(hwnd) {
        error!("{error}");
      }
    }
  }

  if let Err(error) = mailbox.send(WindowMessage::new(hwnd, message, w_param, l_param)) {
    error!("{error}")
  };

  match message {
    WM_CLOSE => LRESULT(0),
    _ => unsafe { DefSubclassProc(hwnd, message, w_param, l_param) },
  }
}
