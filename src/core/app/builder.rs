use crate::debug::validation::ValidationLayer;

use super::App;

pub struct HasTitle(&'static str);
pub struct MissingTitle;

pub struct HasSize(i32, i32);
pub struct MissingSize;

pub struct AppBuilder<Title, Size> {
    title: Title,
    size: Size,
    dark_mode: bool,
}

impl Default for AppBuilder<HasTitle, HasSize> {
    fn default() -> Self {
        Self {
            title: HasTitle("Ookami Renderer"),
            size: HasSize(800, 450),
            dark_mode: true,
        }
    }
}

impl AppBuilder<MissingTitle, MissingSize> {
    pub fn new() -> Self {
        Self {
            title: MissingTitle,
            size: MissingSize,
            dark_mode: true,
        }
    }
}

impl<Size> AppBuilder<MissingTitle, Size> {
    pub fn with_title(self, title: &'static str) -> AppBuilder<HasTitle, Size> {
        AppBuilder {
            title: HasTitle(title),
            size: self.size,
            dark_mode: self.dark_mode,
        }
    }
}

impl<Title> AppBuilder<Title, MissingSize> {
    pub fn with_size(self, width: i32, height: i32) -> AppBuilder<Title, HasSize> {
        AppBuilder {
            title: self.title,
            size: HasSize(width, height),
            dark_mode: self.dark_mode,
        }
    }
}

impl<Title, Size> AppBuilder<Title, Size> {
    pub fn with_dark_mode(self, dark_mode: bool) -> Self {
        Self {
            title: self.title,
            size: self.size,
            dark_mode,
        }
    }
}

impl AppBuilder<HasTitle, HasSize> {
    pub fn run(self) {
        ValidationLayer::instance().init();

        if let Ok(app) = App::new(self.size.0, self.size.1, self.title.0, self.dark_mode) {
            app.run();
        }

        ValidationLayer::instance().shutdown();
    }
}
