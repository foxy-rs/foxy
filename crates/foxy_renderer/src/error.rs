use thiserror::Error;

#[derive(Error, Debug)]
pub enum RendererError {
  #[error("{0}")]
  Error(String),
  #[error("{0}")]
  GlutinError(#[from] glium::glutin::error::Error),
  #[error("{0}")]
  DrawError(#[from] glium::DrawError),
  #[error("{0}")]
  ReadError(#[from] glium::ReadError),
  #[error("{0}")]
  UuidError(#[from] glium::UuidError),
  #[error("{0}")]
  SwapBuffersError(#[from] glium::SwapBuffersError),
  #[error("{0}")]
  ProgramCreationError(#[from] glium::ProgramCreationError),
}

#[macro_export]
macro_rules! renderer_error {
  () => {
    $crate::error::RendererError::Error("renderer error".to_string())
  };
  ($($arg:tt)*) => {{
    $crate::error::RendererError::Error(format!($($arg)*))
  }}
}
