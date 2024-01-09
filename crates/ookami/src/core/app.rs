use ezwin::prelude::*;
use tracing::*;

use self::{
    builder::{AppCreateInfo, HasSize, HasTitle},
    state::AppState,
};

pub mod builder;
mod state;

struct App {
    state: AppState,
    window: Option<Window>,
}

impl App {
    pub fn new(app_create_info: AppCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
        let state = AppState::new();
        let window = Some(
            WindowBuilder::new()
                .with_title(app_create_info.title.0)
                .with_size(app_create_info.size.width, app_create_info.size.height)
                .with_color_mode(app_create_info.color_mode)
                .with_close_behavior(app_create_info.close_behavior)
                .build()?,
        );
        Ok(Self { state, window })
    }

    fn game_loop(&mut self, window: &mut Window, message: &WindowMessage) {
        self.state.time.update();
        self.state.early_update(window, message);
        while self.state.time.should_do_tick() {
            self.state.time.tick();
            self.state.fixed_update(window);
        }
        self.state.update(window, message);
    }

    fn main_loop(mut self) -> anyhow::Result<()> {
        if let Some(mut window) = self.window.take() {
            // window.set_visibility(Visibility::Shown);

            // Main lifecycle
            self.state.start(&mut window);
            while let Some(message) = window.next() {
                self.game_loop(&mut window, &message);
            }
            self.state.stop(&mut window);
        };

        Ok(())
    }

    pub fn run(self) {
        trace!("uchi uchi, uchi da yo");
        if let Err(error) = self.main_loop() {
            error!("{error}");
        }
        trace!("otsu mion");
    }
}
