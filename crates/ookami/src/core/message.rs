pub enum RenderLoopMessage {
    SyncWithGame,
}

pub enum GameLoopMessage {
    SyncWithRenderer,
    RenderData {},
    Exit,
}