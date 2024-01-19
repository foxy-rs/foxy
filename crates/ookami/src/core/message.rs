pub enum RenderLoopMessage {
    SyncWithGame,
}

pub enum GameLoopMessage {
    SyncWithRenderer,
    Exit,
}