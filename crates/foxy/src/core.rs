use self::{
  builder::{FoxyBuilder, FoxyCreateInfo, HasSize, HasTitle, MissingSize, MissingTitle},
  lifecycle::Stage,
  render_loop::RenderLoop,
};
use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_types::{thread::EngineThread, window::Polling};
use foxy_util::{
  log::LogErr,
  time::{EngineTime, Time},
};
use foxy_window::prelude::*;
use message::{GameLoopMessage, RenderLoopMessage};
use messaging::Mailbox;
use std::{
  mem,
  sync::{Arc, Barrier},
};
use tracing::*;

pub mod builder;
pub mod lifecycle;
mod message;
mod render_loop;

pub struct Foxy {
  polling_strategy: Polling,
  render_thread: EngineThread<RenderLoop>,
  game_mailbox: Mailbox<GameLoopMessage, RenderLoopMessage>,
  sync_barrier: Arc<Barrier>,

  current_stage: Stage,
  time: EngineTime,
  window: Window,
}

impl Foxy {
  pub fn builder() -> FoxyBuilder<MissingTitle, MissingSize> {
    Default::default()
  }

  pub(crate) fn new(create_info: FoxyCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
    trace!("Firing up Foxy");

    let time = EngineTime::new(128.0, 1024);

    let mut window = Window::builder()
      .with_title(create_info.title.0)
      .with_size(create_info.size.width, create_info.size.height)
      .with_color_mode(create_info.color_mode)
      .with_close_behavior(create_info.close_behavior)
      .with_visibility(Visibility::Hidden)
      .build()?;

    let renderer = Renderer::new(&window)?;
    window.set_visibility(Visibility::Shown);

    let sync_barrier = Arc::new(Barrier::new(2));

    let (renderer_mailbox, game_mailbox) = Mailbox::new_entangled_pair();
    let render_thread = EngineThread::new(RenderLoop {
      renderer,
      messenger: renderer_mailbox,
      sync_barrier: sync_barrier.clone(),
    });

    let current_state = Stage::Initializing;

    Ok(Self {
      time,
      current_stage: current_state,
      window,
      render_thread,
      game_mailbox,
      sync_barrier,
      polling_strategy: create_info.polling_strategy,
    })
  }

  pub fn time(&self) -> &Time {
    self.time.time()
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  // pub fn poll(&mut self) -> Option<Lifecycle> {
  //   self.next_state(false)
  // }

  // pub fn wait(&mut self) -> Option<Lifecycle> {
  //   self.next_state(true)
  // }

  fn next_window_message(&mut self) -> Option<WindowMessage> {
    if let Polling::Wait = self.polling_strategy {
      self.window.wait()
    } else {
      self.window.next()
    }
  }

  fn next_state(&mut self) -> Option<Stage> {
    let new_state = match &mut self.current_stage {
      Stage::Initializing => {
        self.render_thread.run(());
        Stage::Start
      }
      Stage::Start => {
        info!("KON KON KITSUNE!");
        let message = self.next_window_message();
        if let Some(message) = message {
          Stage::BeginFrame { message }
        } else {
          Stage::Exiting
        }
      }
      Stage::BeginFrame { message } => {
        self.sync_barrier.wait();

        let message = mem::take(message);
        Stage::EarlyUpdate { message }
      }
      Stage::EarlyUpdate { message } => {
        let message = mem::take(message);
        self.time.update();
        if self.time.should_do_tick() {
          self.time.tick();
          Stage::FixedUpdate { message }
        } else {
          Stage::Update { message }
        }
      }
      Stage::FixedUpdate { message } => {
        let message = mem::take(message);
        if self.time.should_do_tick() {
          self.time.tick();
          Stage::FixedUpdate { message }
        } else {
          Stage::Update { message }
        }
      }
      Stage::Update { message } => {
        let message = mem::take(message);
        Stage::EndFrame { message }
      }
      Stage::EndFrame { .. } => {
        let _ = self
          .game_mailbox
          .send(GameLoopMessage::RenderData(RenderData {}))
          .log_error();
        self.sync_barrier.wait();

        let message = self.next_window_message();
        if let Some(message) = message {
          Stage::BeginFrame { message }
        } else {
          Stage::Exiting
        }
      }
      Stage::Exiting => {
        let _ = self.game_mailbox.send(GameLoopMessage::Exit).log_error();
        self.sync_barrier.wait();

        self.render_thread.join();
        Stage::ExitLoop
      }
      Stage::ExitLoop => {
        info!("OTSU KON DESHITA!");
        // self.window.exit();
        return None;
      }
    };

    self.current_stage = new_state;

    // debug!("{:?}", self.current_state);
    Some(self.current_stage.clone())
  }
}

impl Iterator for Foxy {
  type Item = Stage;

  fn next(&mut self) -> Option<Self::Item> {
    self.next_state()
  }
}
