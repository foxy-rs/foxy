use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WindowError {
  #[error("{0}")]
  IOError(#[from] io::Error),
  #[error("{0}")]
  Other(String),
  #[error("unspecified interpreter error")]
  Unspecified,
  #[error("feature not implemented")]
  Unimplemented,
}
