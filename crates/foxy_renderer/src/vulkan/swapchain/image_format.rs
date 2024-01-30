#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ImageFormat {
  pub present_mode: PresentMode,
  pub color_space: ColorSpace,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
  Unorm,
  #[default]
  SRGB,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PresentMode {
  AutoAdaptive,
  AutoImmediate,
  #[default]
  AutoVsync,
}
