use foxy_renderer::error::RendererError;
use foxy_utils::thread::error::ThreadError;
use thiserror::Error;

pub mod builder;
pub mod state;
pub mod framework;
pub mod message;
pub mod runnable;

pub type FoxyResult<T> = Result<T, FoxyError>;

#[derive(Debug, Error)]
pub enum FoxyError {
  #[error("{0}")]
  Error(String),
  #[error("{0}")]
  RendererError(#[from] RendererError),
  #[error("{0}")]
  ThreadError(#[from] ThreadError),
  #[error("{0}")]
  IOError(#[from] std::io::Error),
  #[error("{0}")]
  EventLoopError(#[from] winit::error::EventLoopError),
  #[error("{0}")]
  ExternalError(#[from] winit::error::ExternalError),
  #[error("{0}")]
  NotSupportedError(#[from] winit::error::NotSupportedError),
  #[error("{0}")]
  OsError(#[from] winit::error::OsError),
  #[error("{0}")]
  HandleError(#[from] winit::raw_window_handle::HandleError),
  #[error("{0}")]
  CursorIconParseError(#[from] winit::window::CursorIconParseError),
}

#[macro_export]
macro_rules! foxy_error {
  () => {
    $crate::core::FoxyError::Error("foxy error".to_string())
  };
  ($($arg:tt)*) => {{
    $crate::core::FoxyError::Error(format!($($arg)*))
  }}
}

// #[macro_export]
// macro_rules! foxy_err {
//   () => {
//     Err($crate::core::FoxyError::Error("foxy error".to_string()))
//   };
//   ($($arg:tt)*) => {{
//     Err($crate::core::FoxyError::Error(format!($($arg)*)))
//   }}
// }