use ezwin::prelude::*;
use tracing::*;

use self::{
    builder::{AppCreateInfo, HasSize, HasTitle},
    state::AppState,
};

use super::{
    message::{GameLoopMessage, GameMessenger, RenderLoopMessage, RendererMessenger},
    renderer::{render_data::RenderData, Renderer},
};

pub mod builder;
mod state;

struct App {
    state: AppState,
    // These are optional to allow take-ing in main loop
    window: Option<Window>,
    renderer: Option<Renderer>,
}

impl App {
    pub fn new(app_create_info: AppCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
        let state = AppState::new();
        let mut window = WindowBuilder::new()
            .with_title(app_create_info.title.0)
            .with_size(app_create_info.size.width, app_create_info.size.height)
            .with_color_mode(app_create_info.color_mode)
            .with_close_behavior(app_create_info.close_behavior)
            .with_visibility(Visibility::Hidden)
            .build()?;
        let renderer = Renderer::new(&window)?;
        window.set_visibility(Visibility::Shown);

        Ok(Self {
            state,
            window: Some(window),
            renderer: Some(renderer),
        })
    }

    fn run_internal(mut self) -> anyhow::Result<()> {
        let (render_message_sender, render_message_receiver) =
            std::sync::mpsc::channel::<RenderLoopMessage>();
        let (game_message_sender, game_message_receiver) =
            std::sync::mpsc::channel::<GameLoopMessage>();
        let renderer_messenger =
            RendererMessenger::new(render_message_sender, game_message_receiver);
        let game_messenger = GameMessenger::new(game_message_sender, render_message_receiver);

        // to allow double mutable borrow
        if let (Some(mut window), Some(renderer)) = (self.window.take(), self.renderer.take()) {
            renderer.render_loop(renderer_messenger)?;
            self.game_loop(&mut window, game_messenger)?;
        };

        Ok(())
    }

    fn game_loop(
        &mut self,
        window: &mut Window,
        mut messenger: GameMessenger,
    ) -> anyhow::Result<()> {
        self.state.start(window);

        while let Some(message) = window.next() {
            messenger.sync_and_receive()?;

            // Main lifecycle
            self.state.time.update();
            self.state.early_update(window, &message);
            while self.state.time.should_do_tick() {
                self.state.time.tick();
                self.state.fixed_update(window);
            }
            self.state.update(window, &message);

            messenger.send_and_sync(GameLoopMessage::RenderData(RenderData { }))?;
        }

        messenger.send(GameLoopMessage::Exit);

        self.state.stop(window);

        Ok(())
    }

    pub fn run(self) {
        trace!("uchi uchi, uchi da yo");
        if let Err(error) = self.run_internal() {
            error!("{error}");
        }
        trace!("otsu mion");
    }
}
