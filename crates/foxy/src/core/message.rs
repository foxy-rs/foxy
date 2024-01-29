use std::time::Duration;

use foxy_renderer::renderer::render_data::RenderData;

#[derive(Debug)]
pub enum RenderLoopMessage {
  Response {
    delta_time: Duration,
    average_delta_time: Duration,
  },
}

#[derive(Debug)]
pub enum GameLoopMessage {
  RenderData(RenderData),
  Exit,
}
