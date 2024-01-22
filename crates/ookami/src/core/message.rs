use std::sync::mpsc::{Receiver, Sender};

use tracing::*;

pub enum RenderLoopMessage {
    SyncWithGame,
}

pub struct RendererMessenger {
    sender: Sender<RenderLoopMessage>,
    reciever: Receiver<GameLoopMessage>,
}

impl RendererMessenger {
    pub fn new(sender: Sender<RenderLoopMessage>, reciever: Receiver<GameLoopMessage>) -> Self {
        Self { sender, reciever }
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
        self.reciever.recv().map_err(|e| e.into())
    }

    pub fn sync_and_recieve(&mut self) -> anyhow::Result<GameLoopMessage> {
        self.send_and_sync(RenderLoopMessage::SyncWithGame)
    }
}

pub enum GameLoopMessage {
    SyncWithRenderer,
    RenderData {},
    Exit,
}

pub struct GameMessenger {
    sender: Sender<GameLoopMessage>,
    reciever: Receiver<RenderLoopMessage>,
}

impl GameMessenger {
    pub fn new(sender: Sender<GameLoopMessage>, reciever: Receiver<RenderLoopMessage>) -> Self {
        Self { sender, reciever }
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
        self.reciever.recv().map_err(|e| e.into())
    }

    pub fn sync_and_recieve(&mut self) -> anyhow::Result<RenderLoopMessage> {
        self.send_and_sync(GameLoopMessage::SyncWithRenderer)
    }
}
