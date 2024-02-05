use std::{
  sync::{Arc, RwLock},
  thread::JoinHandle,
  time::Duration,
};

use crossbeam::{channel::TryRecvError, queue::ArrayQueue};
use foxy_renderer::{
  renderer::{render_data::RenderData, Renderer},
  vulkan::Vulkan,
};
use foxy_utils::{
  log::LogErr,
  mailbox::{Mailbox, MessagingError},
  thread::handle::LoopHandle,
  time::{timer::Timer, EngineTime, Time},
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
  FoxyError,
  FoxyResult,
};
use crate::{
  core::message::{GameLoopMessage, RenderLoopMessage},
  foxy_error,
};

pub struct Framework<T: 'static + Send + Sync> {
  polling_strategy: Polling,
  debug_info: DebugInfo,

  event_loop: EventLoop<T>,
  original_title: String,
  window: Arc<Window>,

  renderer: Renderer<Vulkan>,
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
    window.set_visible(true);
    let render_time = create_info.time.build();
    let render_queue = Arc::new(ArrayQueue::new(Self::MAX_FRAME_DATA_IN_FLIGHT));

    let time = create_info.time.build();
    let mut foxy = Foxy::new(time, window.clone());
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
    self.render_mailbox.send(RenderLoopMessage::Start).log_error();

    self.event_loop.set_control_flow(match self.polling_strategy {
      Polling::Poll => ControlFlow::Poll,
      Polling::Wait => ControlFlow::Wait,
    });

    Ok(self.event_loop.run(move |event, elwt| {
      match &event {
        Event::WindowEvent { window_id: _, event } => match event {
          WindowEvent::CloseRequested => {
            let response = self
              .render_mailbox
              .send_and_recv(RenderLoopMessage::ExitRequested)
              .log_error();
            if let Err(_) | Ok(GameLoopMessage::Exit) = response {
              if let Some(thread) = self.game_thread.take() {
                thread.join();
              }
              self.renderer.delete();
              elwt.exit();
            }
          }
          WindowEvent::RedrawRequested => {
            self.render_time.update();
            while self.render_time.should_do_tick_unchecked() {
              self.render_time.tick();
            }

            let render_data = self.render_queue.pop();
            if let Err(error) = self.renderer.draw(self.render_time.time(), render_data) {
              error!("`{error}` Aborting...");
              let _ = self.render_mailbox.send_and_recv(RenderLoopMessage::MustExit);
              elwt.exit();
            }

            if self.fps_timer.has_elapsed(Duration::from_millis(300)) {
              if let DebugInfo::Shown = self.debug_info {
                let time = self.render_time.time();
                let ft = 1.0 / time.average_delta_secs();
                self
                  .window
                  .set_title(&format!("{} | {:.6}", self.original_title, ft,));
              }
            }
          }
          _ => (),
        },
        Event::AboutToWait => {
          if let Polling::Poll = self.polling_strategy {
            self.window.request_redraw();
          }
        }
        Event::LoopExiting => {
          info!("OTSU KON DESHITA!");
        }
        _ => (),
      }

      if !elwt.exiting() {
        self.render_mailbox.send(RenderLoopMessage::Winit(event)).log_error();
      }
    })?)
  }

  fn game_loop<App: Runnable<T>>(
    mut mailbox: Mailbox<GameLoopMessage, RenderLoopMessage<T>>,
    mut foxy: Foxy,
    render_queue: Arc<ArrayQueue<RenderData>>,
  ) -> FoxyResult<JoinHandle<FoxyResult<()>>> {
    let handle = std::thread::Builder::new()
      .name(Self::GAME_THREAD_ID.into())
      .spawn(move || -> FoxyResult<()> {
        mailbox.recv().log_error(); // wait for startup message
        debug!("HELLO HELLO BAU BAU");

        let mut app = App::new(&mut foxy);
        app.start(&mut foxy);
        loop {
          let next_message = mailbox.try_recv();

          let event = match next_message {
            Ok(RenderLoopMessage::MustExit) => {
              mailbox.send(GameLoopMessage::Exit);
              app.stop(&mut foxy);
              app.delete();
              break;
            }
            Ok(RenderLoopMessage::ExitRequested) => {
              if app.stop(&mut foxy) {
                mailbox.send(GameLoopMessage::Exit);
                app.delete();
                break;
              } else {
                mailbox.send(GameLoopMessage::DontExit);
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

  // fn next_window_message(&mut self) -> Option<WindowMessage> {
  //   let message = if let Polling::Wait = self.polling_strategy {
  //     self.foxy.window_mut().wait()
  //   } else {
  //     self.foxy.window_mut().next()
  //   };

  //   // if let Some(WindowMessage::CloseRequested) = message {
  //   //   self.foxy.window_mut().close();
  //   // }

  //   if let Some(WindowMessage::Closing) = message {
  //     let _ = self.game_mailbox.send(GameLoopMessage::Exit).log_error();
  //     self.render_thread.join();
  //   }

  //   message
  // }

  // fn next_state(&mut self) -> Option<Stage<'_>> {
  //   /*
  //    * NOTE: each stage in the match is the PREVIOUS stage!!!
  //    * I've written the ACTUAL stage at the top of each
  //    */
  //   let new_state = match self.current_stage {
  //     StageDiscriminants::Initialize => {
  //       // Start
  //       info!("KON KON KITSUNE!");
  //       self.render_thread.run();
  //       Stage::Start { foxy: &mut self.foxy }
  //     }
  //     StageDiscriminants::Start => {
  //       // Begin Frame / Exiting
  //       if let Some(message) = self.next_window_message() {
  //         self.current_message = message;
  //         Stage::BeginFrame {
  //           foxy: &mut self.foxy,
  //           message: &mut self.current_message,
  //         }
  //       } else {
  //         Stage::Exiting { foxy: &mut self.foxy }
  //       }
  //     }
  //     StageDiscriminants::BeginFrame => {
  //       // Early Update
  //       if matches!(self.current_message, WindowMessage::Closing) {
  //         Stage::EarlyUpdate {
  //           foxy: &mut self.foxy,
  //           message: &mut self.current_message,
  //         }
  //       } else {
  //         match
  // self.game_mailbox.send_and_wait(GameLoopMessage::Sync).log_error() {
  //           Ok(render_response) => match render_response {
  //             RenderLoopMessage::EmergencyExit => Stage::Exiting { foxy: &mut
  // self.foxy },             _ => {
  //               self.foxy.time.update();

  //               Stage::EarlyUpdate {
  //                 foxy: &mut self.foxy,
  //                 message: &mut self.current_message,
  //               }
  //             }
  //           },
  //           Err(_) => Stage::Exiting { foxy: &mut self.foxy },
  //         }
  //       }
  //     }
  //     StageDiscriminants::EarlyUpdate => {
  //       // Fixed Update / Update
  //       if self.foxy.time.should_do_tick_unchecked() {
  //         self.foxy.time.tick();
  //         Stage::FixedUpdate { foxy: &mut self.foxy }
  //       } else {
  //         Stage::Update {
  //           foxy: &mut self.foxy,
  //           message: &mut self.current_message,
  //         }
  //       }
  //     }
  //     StageDiscriminants::FixedUpdate => {
  //       // Fixed Update / Update
  //       if self.foxy.time.should_do_tick_unchecked() {
  //         self.foxy.time.tick();
  //         Stage::FixedUpdate { foxy: &mut self.foxy }
  //       } else {
  //         Stage::Update {
  //           foxy: &mut self.foxy,
  //           message: &mut self.current_message,
  //         }
  //       }
  //     }
  //     StageDiscriminants::Update => {
  //       // End Frame
  //       if matches!(self.current_message, WindowMessage::Closing) {
  //         Stage::EndFrame {
  //           foxy: &mut self.foxy,
  //           message: &mut self.current_message,
  //           render_response: RenderLoopMessage::None,
  //         }
  //       } else {
  //         match self
  //           .game_mailbox
  //           .send_and_wait(GameLoopMessage::RenderInfo {})
  //           .log_error()
  //         {
  //           Ok(render_response) => match render_response {
  //             RenderLoopMessage::EmergencyExit => Stage::Exiting { foxy: &mut
  // self.foxy },             _ => {
  //               // show FPS in window title
  //               if self.fps_timer.has_elapsed(Duration::from_millis(300)) {
  //                 if let DebugInfo::Shown = self.debug_info {
  //                   let ft = self.foxy.time().average_delta_secs();
  //                   self
  //                     .foxy
  //                     .window()
  //                     .set_title(&format!("{} | {:.6} s",
  //                                              self.foxy.window().title(),
  // ft,));                 }               }
  //               Stage::EndFrame {
  //                 foxy: &mut self.foxy,
  //                 message: &mut self.current_message,
  //                 render_response,
  //               }
  //             }
  //           },
  //           Err(_) => Stage::Exiting { foxy: &mut self.foxy },
  //         }
  //       }
  //     }
  //     StageDiscriminants::EndFrame => {
  //       // Begin Frame / Exiting
  //       if let Some(message) = self.next_window_message() {
  //         self.current_message = message;
  //         Stage::BeginFrame {
  //           foxy: &mut self.foxy,
  //           message: &mut self.current_message,
  //         }
  //       } else {
  //         Stage::Exiting { foxy: &mut self.foxy }
  //       }
  //     }
  //     StageDiscriminants::Exiting => {
  //       // Anything
  //       Stage::ExitLoop
  //     }
  //     StageDiscriminants::ExitLoop => {
  //       // Never gets sent to clients
  //       info!("OTSU KON DESHITA!");
  //       return None;
  //     }
  //   };

  //   self.current_stage = StageDiscriminants::from(&new_state);

  //   Some(new_state)
  // }
}
