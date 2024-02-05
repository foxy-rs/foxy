use winit::event::Event;

#[derive(Debug)]
pub enum RenderLoopMessage<T: 'static + Send> {
  Start,
  MustExit,
  ExitRequested,
  Winit(Event<T>),
  None,
}

#[derive(Debug)]
pub enum GameLoopMessage {
  Exit,
  DontExit,
}
