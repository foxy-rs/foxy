use foxy_renderer::renderer::render_data::RenderData;

#[derive(Debug)]
pub enum RenderLoopMessage {
  
}

#[derive(Debug)]
pub enum GameLoopMessage {
  RenderData(RenderData),
  Exit,
}
