use std::sync::mpsc::{Receiver, Sender};

use tracing::*;

use super::renderer::render_data::RenderData;

pub enum RenderLoopMessage {
    SyncWithGame,
}

pub struct RendererMessenger {
    sender: Sender<RenderLoopMessage>,
    receiver: Receiver<GameLoopMessage>,
}

impl RendererMessenger {
    pub fn new(sender: Sender<RenderLoopMessage>, receiver: Receiver<GameLoopMessage>) -> Self {
        Self { sender, receiver }
    }

    pub fn send(&mut self, message: RenderLoopMessage) {
        if let Err(error) = self.sender.send(message) {
            error!("{error}");
        }
    }

    pub fn send_and_sync(&mut self, message: RenderLoopMessage) -> anyhow::Result<GameLoopMessage> {
        if let Err(error) = self.sender.send(message) {
            error!("{error}");
        }
        self.receiver.recv().map_err(|e| e.into())
    }

    pub fn sync_and_receive(&mut self) -> anyhow::Result<GameLoopMessage> {
        self.send_and_sync(RenderLoopMessage::SyncWithGame)
    }
}

pub enum GameLoopMessage {
    SyncWithRenderer,
    RenderData(RenderData),
    Exit,
}

pub struct GameMessenger {
    sender: Sender<GameLoopMessage>,
    receiver: Receiver<RenderLoopMessage>,
}

impl GameMessenger {
    pub fn new(sender: Sender<GameLoopMessage>, receiver: Receiver<RenderLoopMessage>) -> Self {
        Self { sender, receiver }
    }

    pub fn send(&mut self, message: GameLoopMessage) {
        if let Err(error) = self.sender.send(message) {
            error!("{error}");
        }
    }

    pub fn send_and_sync(&mut self, message: GameLoopMessage) -> anyhow::Result<RenderLoopMessage> {
        if let Err(error) = self.sender.send(message) {
            error!("{error}");
        }
        self.receiver.recv().map_err(|e| e.into())
    }

    pub fn sync_and_receive(&mut self) -> anyhow::Result<RenderLoopMessage> {
        self.send_and_sync(GameLoopMessage::SyncWithRenderer)
    }
}
