use std::time::Duration;

#[derive(Debug)]
pub enum RenderLoopMessage {
  EmergencyExit,
  Response {
    delta_time: Duration,
    average_delta_time: Duration,
  },
  None,
}

#[derive(Debug)]
pub enum GameLoopMessage {
  Sync,
  RenderInfo {},
  Exit,
}
