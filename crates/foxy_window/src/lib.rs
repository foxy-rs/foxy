#![cfg_attr(target_os, windows)] // for now, it only supports Win32
#![deny(unsafe_op_in_unsafe_fn)]

pub mod debug;
pub mod prelude;
pub mod window;

pub fn loword(dword: u32) -> u16 {
  dword as u16
}

pub fn hiword(dword: u32) -> u16 {
  (dword >> 16) as u16
}

pub fn lobyte(word: u16) -> u8 {
  word as u8
}

pub fn hibyte(word: u16) -> u8 {
  (word >> 8) as u8
}
