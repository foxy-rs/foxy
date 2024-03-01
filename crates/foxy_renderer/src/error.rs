use thiserror::Error;

#[derive(Error, Debug)]
pub enum RendererError {
  #[error("{0}")]
  Error(String),
}

#[macro_export]
macro_rules! renderer_error {
  () => {{
    $crate::error::RendererError::Error("renderer error".to_string())
  }};
  ($($arg:tt)*) => {{
    $crate::error::RendererError::Error(format!($($arg)*))
  }}
}
