use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WindowError {
  #[error("{0}")]
  IOError(#[from] io::Error),
  #[error("{0}")]
  Win32Error(#[from] windows::core::Error),
  #[error("{0}")]
  Anyhow(#[from] anyhow::Error),
  #[error("{0}")]
  Other(String),
}
