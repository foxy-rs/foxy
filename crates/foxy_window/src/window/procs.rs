use std::sync::mpsc::Sender;

use foxy_util::log::LogErr;
use windows::Win32::{
  Foundation::*,
  UI::{Shell::DefSubclassProc, WindowsAndMessaging::*},
};

use super::message::WindowMessage;

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
  let sender: &mut Sender<WindowMessage> = unsafe { std::mem::transmute(dw_ref_data) };

  let _ = sender
    .send(WindowMessage::new(hwnd, message, w_param, l_param))
    .log_error_msg("WindowMessage::new");

  match message {
    WM_CLOSE => LRESULT(0),
    WM_DESTROY => {
      unsafe {
        PostQuitMessage(0);
      }
      LRESULT(0)
    }
    _ => unsafe { DefSubclassProc(hwnd, message, w_param, l_param) },
  }
}
