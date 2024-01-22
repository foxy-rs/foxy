use std::sync::mpsc::{Receiver, Sender};

use tracing::*;

use super::renderer::render_data::RenderData;

#[derive(Debug)]
pub enum RenderLoopMessage {
    SyncWithGame,
}

#[derive(Debug)]
pub enum GameLoopMessage {
    SyncWithRenderer,
    RenderData(RenderData),
    Exit,
}
