use windows::Win32::Foundation::HWND;

pub enum Message {
    WindowCreated { hwnd: HWND },
    WindowClosed,
}
