use ezwin::prelude::Message;

#[derive(Debug)]
pub enum RenderLoopMessage {
  Start,
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
