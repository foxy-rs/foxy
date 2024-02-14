use thiserror::Error;

#[derive(Error, Debug)]
pub enum RendererError {
  #[error("{0}")]
  Error(String),
  #[error("{0}")]
  WgpuError(#[from] wgpu::Error),
  #[error("{0}")]
  SurfaceError(#[from] wgpu::SurfaceError),
  #[error("must rebuild swapchain")]
  RebuildSwapchain,
  #[error("{0}")]
  CreateSurfaceError(#[from] wgpu::CreateSurfaceError),
  #[error("{0}")]
  RequestDeviceError(#[from] wgpu::RequestDeviceError),
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
