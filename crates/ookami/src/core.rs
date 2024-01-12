pub mod app;
mod time;

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