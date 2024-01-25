// use foxy_window::window::builder::{CloseBehavior, ColorMode, Visibility};

// use super::Renderer;

// #[derive(Debug, Clone)]
// pub struct HasTitle(pub &'static str);
// pub struct MissingTitle;

// #[derive(Debug, Clone)]
// pub struct HasSize {
//   pub width: i32,
//   pub height: i32,
// }
// pub struct MissingSize;

// #[derive(Debug, Clone)]
// pub struct CanvasCreateInfo<Title, Size> {
//   pub title: Title,
//   pub size: Size,
//   pub color_mode: ColorMode,
//   pub close_behavior: CloseBehavior,
//   pub visibility: Visibility,
// }

// pub struct CanvasBuilder<Title, Size> {
//   create_info: CanvasCreateInfo<Title, Size>,
// }

// impl CanvasBuilder<MissingTitle, MissingSize> {
//   pub fn new() -> Self {
//     Self::default()
//   }
// }

// impl Default for CanvasBuilder<MissingTitle, MissingSize> {
//   fn default() -> Self {
//     Self {
//       create_info: CanvasCreateInfo {
//         title: MissingTitle,
//         size: MissingSize,
//         color_mode: ColorMode::Dark,
//         close_behavior: CloseBehavior::Default,
//         visibility: Visibility::Shown,
//       },
//     }
//   }
// }

// impl<Size> CanvasBuilder<MissingTitle, Size> {
//   pub fn with_title(self, title: &'static str) -> CanvasBuilder<HasTitle, Size> {
//     CanvasBuilder {
//       create_info: CanvasCreateInfo {
//         title: HasTitle(title),
//         size: self.create_info.size,
//         color_mode: self.create_info.color_mode,
//         close_behavior: self.create_info.close_behavior,
//         visibility: self.create_info.visibility,
//       },
//     }
//   }
// }

// impl<Title> CanvasBuilder<Title, MissingSize> {
//   pub fn with_size(self, width: i32, height: i32) -> CanvasBuilder<Title, HasSize> {
//     CanvasBuilder {
//       create_info: CanvasCreateInfo {
//         title: self.create_info.title,
//         size: HasSize { width, height },
//         color_mode: self.create_info.color_mode,
//         close_behavior: self.create_info.close_behavior,
//         visibility: self.create_info.visibility,
//       },
//     }
//   }
// }

// impl<Title, Size> CanvasBuilder<Title, Size> {
//   pub fn with_color_mode(self, color_mode: ColorMode) -> Self {
//     Self {
//       create_info: CanvasCreateInfo {
//         title: self.create_info.title,
//         size: self.create_info.size,
//         color_mode,
//         close_behavior: self.create_info.close_behavior,
//         visibility: self.create_info.visibility,
//       },
//     }
//   }

//   pub fn with_close_behavior(self, close_behavior: CloseBehavior) -> Self {
//     Self {
//       create_info: CanvasCreateInfo {
//         title: self.create_info.title,
//         size: self.create_info.size,
//         color_mode: self.create_info.color_mode,
//         close_behavior,
//         visibility: self.create_info.visibility,
//       },
//     }
//   }

//   pub fn with_visibility(self, visibility: Visibility) -> Self {
//     Self {
//       create_info: CanvasCreateInfo {
//         title: self.create_info.title,
//         size: self.create_info.size,
//         color_mode: self.create_info.color_mode,
//         close_behavior: self.create_info.close_behavior,
//         visibility,
//       },
//     }
//   }
// }

// impl CanvasBuilder<HasTitle, HasSize> {
//   pub fn build(self) -> anyhow::Result<Renderer> {
//     Renderer::new(self.create_info)
//   }
// }
