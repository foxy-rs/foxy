use strum::Display;
use windows::Win32::Foundation::HWND;

#[derive(Debug, Display)]
pub enum Message {
    Empty,
    WindowOpened { hwnd: HWND },
    WindowClosed,
    KeyDown { },
    KeyUp { },
    Exit,
}
