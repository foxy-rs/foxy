use std::{marker::PhantomData, time::Duration};

use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_utils::{
  log::LogErr,
  thread::handle::LoopHandle,
  time::{timer::Timer, EngineTime},
  types::behavior::Polling,
};
use foxy_window::{prelude::*, window::window_message::StateMessage};
use messaging::Mailbox;
use tracing::*;

use super::{builder::DebugInfo, engine::Foxy, stage::StageDiscriminants};
use crate::core::{
  builder::{FoxyBuilder, FoxyCreateInfo, HasSize, HasTitle, MissingSize, MissingTitle},
  message::{GameLoopMessage, RenderLoopMessage},
  render_loop::RenderLoop,
  stage::Stage,
};

pub struct Framework<'a> {
  polling_strategy: Polling,
  debug_info: DebugInfo,

  render_thread: LoopHandle<RenderLoop, ()>,
  game_mailbox: Mailbox<GameLoopMessage, RenderLoopMessage>,

  current_stage: StageDiscriminants,
  current_message: WindowMessage,

  foxy: Foxy,
  fps_timer: Timer,

  _phantom: PhantomData<&'a ()>,
}

impl Framework<'_> {
  pub const RENDER_THREAD_ID: &'static str = "render";

  pub fn builder() -> FoxyBuilder<MissingTitle, MissingSize> {
    Default::default()
  }

  pub(crate) fn new(create_info: FoxyCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
    trace!("Firing up Foxy");

    // TODO: make this adjustable
    let time = EngineTime::default();
    let render_time = EngineTime::default();

    let mut window = Window::builder()
      .with_title(create_info.title.0)
      .with_size(create_info.size.width, create_info.size.height)
      .with_color_mode(create_info.color_mode)
      .with_visibility(Visibility::Hidden)
      .build()?;

    let renderer = Renderer::new(&window, window.inner_size())?;
    window.set_visibility(Visibility::Shown);

    let (renderer_mailbox, game_mailbox) = Mailbox::new_entangled_pair();
    let render_thread = LoopHandle::new(
      vec![Self::RENDER_THREAD_ID.into()],
      RenderLoop {
        renderer,
        messenger: renderer_mailbox,
        time: render_time,
        should_exit: false,
      },
      (),
    );

    let current_stage = StageDiscriminants::Initialize;
    let foxy = Foxy::new(time, window);

    Ok(Self {
      current_stage,
      render_thread,
      game_mailbox,
      polling_strategy: create_info.polling_strategy,
      debug_info: create_info.debug_info,
      foxy,
      fps_timer: Timer::new(),
      current_message: WindowMessage::None,
      _phantom: PhantomData,
    })
  }

  pub fn foxy(&mut self) -> &mut Foxy {
    &mut self.foxy
  }

  fn close(&mut self) {
    let _ = self.game_mailbox.send(GameLoopMessage::Exit).log_error();
    self.render_thread.join();
    self.foxy.window_mut().close();
  }

  fn next_window_message(&mut self) -> Option<WindowMessage> {
    if let Polling::Wait = self.polling_strategy {
      match self.foxy.window_mut().wait() {
        Some(message) => {
          if let WindowMessage::State(StateMessage::CloseRequested) = &message {
            self.close();
            None
          } else {
            Some(message)
          }
        }
        None => None,
      }
    } else {
      match self.foxy.window_mut().next() {
        Some(message) => {
          if let WindowMessage::State(StateMessage::CloseRequested) = &message {
            self.close();
            None
          } else {
            Some(message)
          }
        }
        None => None,
      }
    }
  }

  fn next_state(&mut self) -> Option<Stage<'_>> {
    /*
     * NOTE: each stage in the match is the PREVIOUS stage!!!
     *       I've written the ACTUAL stage at the top of each
     */
    let new_state = match self.current_stage {
      StageDiscriminants::Initialize => {
        // Start
        info!("KON KON KITSUNE!");
        self.render_thread.run();
        Stage::Start { foxy: &mut self.foxy }
      }
      StageDiscriminants::Start => {
        // Begin Frame / Exiting
        if let Some(message) = self.next_window_message() {
          self.current_message = message;
          Stage::BeginFrame {
            foxy: &mut self.foxy,
            message: &mut self.current_message,
          }
        } else {
          Stage::Exiting { foxy: &mut self.foxy }
        }
      }
      StageDiscriminants::BeginFrame => {
        // Early Update
        match self.game_mailbox.send_and_wait(GameLoopMessage::Sync).log_error() {
          Ok(render_response) => match render_response {
            RenderLoopMessage::EmergencyExit => Stage::Exiting { foxy: &mut self.foxy },
            RenderLoopMessage::Response { .. } => {
              // self.sync_barrier.wait();
              self.foxy.time.update();

              Stage::EarlyUpdate {
                foxy: &mut self.foxy,
                message: &mut self.current_message,
              }
            }
          },
          Err(_) => Stage::Exiting { foxy: &mut self.foxy },
        }
      }
      StageDiscriminants::EarlyUpdate => {
        // Fixed Update / Update
        if self.foxy.time.should_do_tick_unchecked() {
          self.foxy.time.tick();
          Stage::FixedUpdate { foxy: &mut self.foxy }
        } else {
          Stage::Update {
            foxy: &mut self.foxy,
            message: &mut self.current_message,
          }
        }
      }
      StageDiscriminants::FixedUpdate => {
        // Fixed Update / Update
        if self.foxy.time.should_do_tick_unchecked() {
          self.foxy.time.tick();
          Stage::FixedUpdate { foxy: &mut self.foxy }
        } else {
          Stage::Update {
            foxy: &mut self.foxy,
            message: &mut self.current_message,
          }
        }
      }
      StageDiscriminants::Update => {
        // End Frame
        match self
          .game_mailbox
          .send_and_wait(GameLoopMessage::RenderData(RenderData {}))
          .log_error()
        {
          Ok(render_response) => match render_response {
            RenderLoopMessage::EmergencyExit => Stage::Exiting { foxy: &mut self.foxy },
            RenderLoopMessage::Response { .. } => {
              if self.fps_timer.has_elapsed(Duration::from_millis(300)) {
                if let DebugInfo::Shown = self.debug_info {
                  let fps = 1.0 / self.foxy.time().average_delta_secs();
                  self
                    .foxy
                    .window()
                    .set_title(&format!("{} | FPS: {:.2}", self.foxy.window().title(), fps,));
                }
              }
              Stage::EndFrame {
                foxy: &mut self.foxy,
                message: &mut self.current_message,
                render_response,
              }
            }
          },
          Err(_) => Stage::Exiting { foxy: &mut self.foxy },
        }
      }
      StageDiscriminants::EndFrame => {
        // Begin Frame / Exiting
        if let Some(message) = self.next_window_message() {
          self.current_message = message;
          Stage::BeginFrame {
            foxy: &mut self.foxy,
            message: &mut self.current_message,
          }
        } else {
          Stage::Exiting { foxy: &mut self.foxy }
        }
      }
      StageDiscriminants::Exiting => {
        // Anything
        Stage::ExitLoop
      }
      StageDiscriminants::ExitLoop => {
        // Never gets sent to clients
        info!("OTSU KON DESHITA!");
        return None;
      }
    };

    self.current_stage = StageDiscriminants::from(&new_state);

    Some(new_state)
  }
}

impl<'a> Iterator for Framework<'a> {
  type Item = Stage<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    // it is irrefutable that the reference not outlive Foxy
    unsafe { std::mem::transmute(self.next_state()) }
  }
}
