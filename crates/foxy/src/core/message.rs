use std::time::Duration;

use foxy_renderer::renderer::render_data::RenderData;

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
  RenderData(RenderData),
  Exit,
}
