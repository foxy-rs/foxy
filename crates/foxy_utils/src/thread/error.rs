use std::io;

use thiserror::Error;

#[allow(unused)]
#[derive(Error, Debug)]
pub enum ThreadError {
  #[error("{0}")]
  IO(#[from] io::Error),
  #[error("{0}")]
  Other(String),
}

#[allow(unused)]
#[macro_export]
macro_rules! thread_error_fmt {
  () => {{
    $crate::thread::error::ThreadError::Other(format!("thread has experienced an error"))
  }};
  ($($arg:tt)*) => {{
    $crate::thread::error::ThreadError::Other(format!($($arg)*))
  }};
}

#[allow(unused)]
#[macro_export]
macro_rules! thread_err {
  ($($arg:tt)*) => {{
    Err($crate::thread_error_fmt!($($arg)*))
  }};
}

#[allow(unused)]
#[macro_export]
macro_rules! thread_error {
  ($arg:tt) => {{
    $crate::thread_error_fmt!("{}", $arg)
  }};
}
