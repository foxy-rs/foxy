use tracing::{error, trace};

use self::{
    state::AppState,
    window::{message::Message, AppWindow, Visibility},
};

pub mod builder;
mod state;
mod window;

struct App {
    state: AppState,
    window: AppWindow,
}

impl App {
    pub fn new(width: i32, height: i32, title: &str, dark_mode: bool) -> anyhow::Result<Self> {
        let state = AppState::new();
        let window = AppWindow::new(width, height, title, dark_mode)?;
        Ok(Self { state, window })
    }

    fn game_loop(&mut self, event: &Message) {
        self.state.time.update();
        self.state.early_update(event);
        while self.state.time.should_do_tick() {
            self.state.time.tick();
            self.state.fixed_update(event);
        }
        self.state.update(event);
    }

    fn main_loop(mut self) -> anyhow::Result<()> {
        self.window.set_visibility(Visibility::Shown);

        // Main lifecycle
        self.state.start();
        while let Some(event) = self.window.next() {
            self.game_loop(&event);
        }
        self.state.stop();

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
