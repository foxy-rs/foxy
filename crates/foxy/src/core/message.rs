use witer::prelude::*;

#[derive(Debug)]
pub enum RenderLoopMessage {
  MustExit,
  ExitRequested,
  Window(Message),
  None,
}

#[derive(Debug)]
pub enum GameLoopMessage {
  Exit,
  DontExit,
}
