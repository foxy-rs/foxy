use winit::event::WindowEvent;

#[derive(Debug)]
pub enum RenderLoopMessage {
  MustExit,
  ExitRequested,
  Window(WindowEvent),
  None,
}

#[derive(Debug)]
pub enum GameLoopMessage {
  Exit,
  DontExit,
}
