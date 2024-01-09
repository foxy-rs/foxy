use windows::Win32::Foundation::HWND;

use super::{
    builder::{CloseBehavior, ColorMode, Visibility},
    input::Input,
};

#[derive(Debug)]
pub struct WindowState {
    pub hwnd: HWND,
    pub width: i32,
    pub height: i32,
    pub title: String,
    pub color_mode: ColorMode,
    pub close_behavior: CloseBehavior,
    pub visibility: Visibility,
    pub input: Input,
}
