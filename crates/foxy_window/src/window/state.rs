use super::input::Input;
use foxy_types::behavior::{CloseBehavior, ColorMode, Visibility};
use windows::Win32::Foundation::{HINSTANCE, HWND};

#[derive(Debug)]
pub struct WindowState {
  pub hwnd: HWND,
  pub hinstance: HINSTANCE,
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
