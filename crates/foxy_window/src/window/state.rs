use foxy_utils::types::{
  behavior::{ColorMode, Visibility},
  primitives::Dimensions,
};

use super::input::Input;

#[derive(Debug)]
pub struct WindowState {
  pub hwnd: isize,
  pub hinstance: isize,
  pub size: Dimensions,
  pub inner_size: Dimensions,
  pub title: String,
  pub color_mode: ColorMode,
  pub visibility: Visibility,
  pub input: Input,
}
