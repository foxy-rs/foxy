use crossbeam::channel::Sender;
use foxy_utils::log::LogErr;
use windows::Win32::{
  Foundation::*,
  UI::{Shell::DefSubclassProc, WindowsAndMessaging::*},
};

use super::window_message::WindowMessage;

pub struct SubclassWindowData {
  pub priority_sender: Sender<WindowMessage>,
  pub sender: Sender<WindowMessage>,
}

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
  let data: &SubclassWindowData = unsafe { std::mem::transmute(dw_ref_data) };

  let win_message = WindowMessage::new(hwnd, message, w_param, l_param);
  let _ = data.sender.send(win_message).log_error();
  // if matches!(
  //   win_message,
  //   WindowMessage::Other { .. } | WindowMessage::Moved | WindowMessage::Resized
  // { .. } ) {
  //   let _ = data.sender.send(win_message).log_error();
  // } else {
  //   let _ = data.priority_sender.send(win_message).log_error();
  // }

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
