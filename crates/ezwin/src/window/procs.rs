use tracing::*;
use windows::Win32::{
    Foundation::*,
    UI::{Shell::DefSubclassProc, WindowsAndMessaging::*},
};

use super::message::{AppMessage, Mailbox, WindowMessage};

pub extern "system" fn wnd_proc(
    hwnd: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
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
    let Mailbox {
        sender,
        receiver,
    } = unsafe { &mut *(dw_ref_data as *mut Mailbox<WindowMessage, AppMessage>) };

    if let Ok(AppMessage::Exit) = receiver.try_recv() {
        if let Err(error) = sender.send(WindowMessage::Exit) {
            error!("{error}")
        };
        unsafe {
            if let Err(error) = DestroyWindow(hwnd) {
                error!("{error}");
            }
        }
    }

    if let Err(error) =
        sender.send(WindowMessage::new(hwnd, message, w_param, l_param))
    {
        error!("{error}")
    };

    match message {
        WM_CLOSE => LRESULT(0),
        _ => unsafe { DefSubclassProc(hwnd, message, w_param, l_param) },
    }
}
