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

// impl WindowState {
//     pub fn new(
//         hwnd: HWND,
//         width: i32,
//         height: i32,
//         title: &str,
//         color_mode: ColorMode,
//         close_behavior: CloseBehavior,
//         visibility: Visibility,
//         input: Input,
//     ) -> Self {
//         Self {
//             hwnd,
//             width,
//             height,
//             title: String::from(title),
//             color_mode,
//             close_behavior,
//             visibility,
//             input,
//         }
//     }
// }
