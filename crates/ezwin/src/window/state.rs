use windows::Win32::Foundation::HWND;

use super::{
    builder::{CloseBehavior, ColorMode, Visibility},
    input::Input,
};

#[derive(Debug)]
pub struct WindowState {
    pub hwnd: HWND,
    pub size: WindowSize,
    pub title: String,
    pub color_mode: ColorMode,
    pub close_behavior: CloseBehavior,
    pub visibility: Visibility,
    pub input: Input,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowSize {
    pub width: i32,
    pub height: i32,
}
