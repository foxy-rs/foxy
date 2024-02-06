use std::{sync::Arc, thread::JoinHandle, time::Duration};

use crossbeam::{channel::TryRecvError, queue::ArrayQueue};
use foxy_renderer::{
  renderer::{render_data::RenderData, Renderer},
};
use foxy_utils::{
  log::LogErr,
  mailbox::{Mailbox, MessagingError},
  time::{timer::Timer, EngineTime},
};
use tracing::*;
use winit::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::Window,
};

use super::{
  builder::{DebugInfo, FoxyCreateInfo, Polling},
  runnable::Runnable,
  state::Foxy,
  FoxyResult,
};
use crate::core::message::{GameLoopMessage, RenderLoopMessage};

pub struct Framework<T: 'static + Send + Sync> {
  polling_strategy: Polling,
  debug_info: DebugInfo,

  event_loop: EventLoop<T>,
  original_title: String,
  window: Arc<Window>,

  renderer: Renderer,
  render_time: EngineTime,
  render_queue: Arc<ArrayQueue<RenderData>>,
  render_mailbox: Mailbox<RenderLoopMessage<T>, GameLoopMessage>,

  game_thread: Option<JoinHandle<FoxyResult<()>>>,

  fps_timer: Timer,
}

impl Framework<()> {
  pub fn new<App: Runnable<()>>(create_info: FoxyCreateInfo) -> FoxyResult<Self> {
    Self::with_events::<App>(create_info)
  }
}

impl<T: 'static + Send + Sync> Framework<T> {
  const GAME_THREAD_ID: &'static str = "foxy";
  const MAX_FRAME_DATA_IN_FLIGHT: usize = 2;

  pub fn with_events<App: Runnable<T>>(create_info: FoxyCreateInfo) -> FoxyResult<Self> {
    trace!("Firing up Foxy");

    let (event_loop, window) = create_info.window.create_window()?;
    let window = Arc::new(window);

    let renderer = Renderer::new(&window, window.inner_size())?;
    let render_time = create_info.time.build();
    let render_queue = Arc::new(ArrayQueue::new(Self::MAX_FRAME_DATA_IN_FLIGHT));

    let time = create_info.time.build();
    let foxy = Foxy::new(time, window.clone());
    let (game_mailbox, render_mailbox) = Mailbox::new_entangled_pair();
    let game_thread = Some(Self::game_loop::<App>(game_mailbox, foxy, render_queue.clone())?);

    Ok(Self {
      polling_strategy: create_info.polling_strategy,
      debug_info: create_info.debug_info,
      event_loop,
      original_title: window.title(),
      window,
      renderer,
      render_time,
      render_queue,
      render_mailbox,
      game_thread,
      fps_timer: Timer::new(),
    })
  }

  pub fn run(mut self) -> FoxyResult<()> {
    info!("KON KON KITSUNE!");
    let _ = self.render_mailbox.send(RenderLoopMessage::Start).log_error();

    self.event_loop.set_control_flow(match self.polling_strategy {
      Polling::Poll => ControlFlow::Poll,
      Polling::Wait => ControlFlow::Wait,
    });

    let mut had_first_frame = false;
    Ok(self.event_loop.run(move |event, elwt| {
      match &event {
        Event::WindowEvent {
          event: WindowEvent::CloseRequested,
          ..
        } => {
          let response = self
            .render_mailbox
            .send_and_recv(RenderLoopMessage::ExitRequested)
            .log_error();
          if let Err(_) | Ok(GameLoopMessage::Exit) = response {
            if let Some(thread) = self.game_thread.take() {
              let _ = thread.join();
            }
            self.renderer.delete();
            elwt.exit();
          }
        }
        Event::AboutToWait => {
          if !elwt.exiting() {
            let render_data = self.render_queue.pop();

            if render_data.is_some() {
              // only update time when a render is going to occur
              self.render_time.update();
              while self.render_time.should_do_tick_unchecked() {
                self.render_time.tick();
              }
            }

            match self.renderer.draw(self.render_time.time(), render_data) {
              Ok(true) if !had_first_frame => {
                had_first_frame = true;
                self.window.set_visible(true);
              }
              Err(error) => {
                error!("`{error}` Aborting...");
                let _ = self.render_mailbox.send_and_recv(RenderLoopMessage::MustExit);
                elwt.exit();
              }
              _ => (),
            }

            if self.fps_timer.has_elapsed(Duration::from_millis(200)) {
              if let DebugInfo::Shown = self.debug_info {
                let time = self.render_time.time();
                let ft = time.average_delta_secs();
                self
                  .window
                  .set_title(&format!("{} | {:^5.4} s | {:>5.0} FPS", self.original_title, ft, 1.0 / ft));
              }
            }
          }
        }
        Event::LoopExiting => {
          info!("OTSU KON DESHITA!");
        }
        _ => (),
      }

      if !elwt.exiting() {
        let _ = self.render_mailbox.send(RenderLoopMessage::Winit(event)).log_error();
      }
    })?)
  }

  fn game_loop<App: Runnable<T>>(
    mailbox: Mailbox<GameLoopMessage, RenderLoopMessage<T>>,
    mut foxy: Foxy,
    render_queue: Arc<ArrayQueue<RenderData>>,
  ) -> FoxyResult<JoinHandle<FoxyResult<()>>> {
    let handle = std::thread::Builder::new()
      .name(Self::GAME_THREAD_ID.into())
      .spawn(move || -> FoxyResult<()> {
        let _ = mailbox.recv().log_error(); // wait for startup message
        debug!("HELLO HELLO BAU BAU");

        let mut app = App::new(&mut foxy);
        app.start(&mut foxy);
        loop {
          let next_message = mailbox.try_recv();

          let event = match next_message {
            Ok(RenderLoopMessage::MustExit) => {
              let _ = mailbox.send(GameLoopMessage::Exit);
              app.stop(&mut foxy);
              app.delete();
              break;
            }
            Ok(RenderLoopMessage::ExitRequested) => {
              if app.stop(&mut foxy) {
                let _ = mailbox.send(GameLoopMessage::Exit);
                app.delete();
                break;
              } else {
                let _ = mailbox.send(GameLoopMessage::DontExit);
              }
              None
            }
            Ok(RenderLoopMessage::Winit(event)) => Some(event),
            Err(MessagingError::TryRecvError {
              error: TryRecvError::Disconnected,
            }) => {
              app.stop(&mut foxy);
              app.delete();
              break;
            }
            _ => None,
          };

          // Loop

          foxy.time.update();
          while foxy.time.should_do_tick_unchecked() {
            foxy.time.tick();
            app.fixed_update(&mut foxy, &event);
          }
          app.update(&mut foxy, &event);
          render_queue.force_push(RenderData {});
        }

        debug!("BAU BAU FOR NOW");
        Ok(())
      })?;

    Ok(handle)
  }
}
