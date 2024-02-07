use winit::event::WindowEvent;


#[derive(Debug)]
pub enum RenderLoopMessage {
  Start,
  MustExit,
  ExitRequested,
  Winit(WindowEvent),
  None,
}

#[derive(Debug)]
pub enum GameLoopMessage {
  Exit,
  DontExit,
}
