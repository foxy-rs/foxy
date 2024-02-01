use windows::Win32::Foundation::HWND;

#[derive(Debug, PartialEq, Eq)]
pub enum MainMessage {
  Empty,
  Close,
  DestroyWindow { hwnd: HWND },
  Exit,
}
