use self::{
  builder::{FoxyBuilder, FoxyCreateInfo, HasSize, HasTitle, MissingSize, MissingTitle},
  lifecycle::Lifecycle,
  render_loop::RenderLoop,
};
use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_types::thread::EngineThread;
use foxy_util::{log::LogErr, time::{EngineTime, Time}};
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
  time: EngineTime,
  current_state: Lifecycle,
  window: Window,
  render_thread: EngineThread<RenderLoop>,
  game_mailbox: Mailbox<GameLoopMessage, RenderLoopMessage>,
  sync_barrier: Arc<Barrier>,
}

impl Foxy {
  pub fn builder() -> FoxyBuilder<MissingTitle, MissingSize> {
    Default::default()
  }

  pub fn new(foxy_create_info: FoxyCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
    info!("kon kon kitsune!");

    let time = EngineTime::new(128.0, 1024);

    let mut window = Window::builder()
      .with_title(foxy_create_info.title.0)
      .with_size(foxy_create_info.size.width, foxy_create_info.size.height)
      .with_color_mode(foxy_create_info.color_mode)
      .with_close_behavior(foxy_create_info.close_behavior)
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

    let current_state = Lifecycle::Initializing;

    Ok(Self {
      time,
      current_state,
      window,
      render_thread,
      game_mailbox,
      sync_barrier,
    })
  }

  pub fn time(&self) -> &Time {
    self.time.time()
  }

  pub fn window(&self) -> &Window {
    &self.window
  }

  pub fn poll(&mut self) -> Option<&Lifecycle> {
    self.next_state(false)
  }

  pub fn wait(&mut self) -> Option<&Lifecycle> {
    self.next_state(true)
  }
  
  fn next_window_message(&mut self, should_wait: bool) -> Option<WindowMessage> {
    if should_wait {
      self.window.wait()
    } else {
      self.window.next()
    }
  }

  fn next_state(&mut self, should_wait: bool) -> Option<&Lifecycle> {
    let new_state = match &mut self.current_state {
      Lifecycle::Initializing => {
        self.render_thread.run(());
        Lifecycle::Start
      }
      Lifecycle::Start => {
        let message = self.next_window_message(should_wait);
        if let Some(message) = message {
          Lifecycle::BeginFrame { message }
        } else {
          Lifecycle::Exiting
        }
      }
      Lifecycle::BeginFrame { message } => {
        self.sync_barrier.wait();

        let message = mem::take(message);
        Lifecycle::EarlyUpdate { message }
      }
      Lifecycle::EarlyUpdate { message } => {
        let message = mem::take(message);
        self.time.update();
        if self.time.should_do_tick() {
          self.time.tick();
          Lifecycle::FixedUpdate { message }
        } else {
          Lifecycle::Update { message }
        }
      }
      Lifecycle::FixedUpdate { message } => {
        let message = mem::take(message);
        if self.time.should_do_tick() {
          self.time.tick();
          Lifecycle::FixedUpdate { message }
        } else {
          Lifecycle::Update { message }
        }
      }
      Lifecycle::Update { message } => {
        let message = mem::take(message);
        Lifecycle::EndFrame { message }
      }
      Lifecycle::EndFrame { .. } => {
        let _ = self.game_mailbox.send(GameLoopMessage::RenderData(RenderData {})).log_error();
        self.sync_barrier.wait();

        let message = self.next_window_message(should_wait);
        if let Some(message) = message {
          Lifecycle::BeginFrame { message }
        } else {
          Lifecycle::Exiting
        }
      }
      Lifecycle::Exiting => {
        let _ = self.game_mailbox.send(GameLoopMessage::Exit).log_error();
        self.sync_barrier.wait();

        self.render_thread.join();
        Lifecycle::ExitLoop
      }
      Lifecycle::ExitLoop => {
        // self.window.exit();
        return None;
      }
    };

    self.current_state = new_state;

    // debug!("{:?}", self.current_state);
    Some(&self.current_state)
  }
}
