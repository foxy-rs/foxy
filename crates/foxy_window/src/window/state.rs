use foxy_utils::types::behavior::{ColorMode, Visibility};

use super::{input::Input, window_message::SizeState};

#[derive(Debug)]
pub struct WindowState {
  pub hwnd: isize,
  pub hinstance: isize,
  pub size_state: SizeState,
  pub title: String,
  pub color_mode: ColorMode,
  pub visibility: Visibility,
  pub input: Input,
}
