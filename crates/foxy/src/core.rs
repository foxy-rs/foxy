use self::{
  builder::{FoxyBuilder, FoxyCreateInfo, HasSize, HasTitle, MissingSize, MissingTitle}, lifecycle::Lifecycle, render_loop::RenderThread
};
use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_util::time::{EngineTime, Time};
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
  render_thread: RenderThread,
  game_mailbox: Mailbox<GameLoopMessage, RenderLoopMessage>,
  sync_barrier: Arc<Barrier>,
}

impl Foxy {
  pub fn builder() -> FoxyBuilder<MissingTitle, MissingSize> {
    Default::default()
  }

  pub fn new(foxy_create_info: FoxyCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
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
    let render_thread = RenderThread::new(renderer, renderer_mailbox, sync_barrier.clone());

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

  fn next_state(&mut self, should_wait: bool) -> Option<&Lifecycle> {
    fn poll_window_or_wait(foxy: &mut Foxy, should_wait: bool) -> Option<WindowMessage> {
      if should_wait {
        foxy.window.wait()
      } else {
        foxy.window.next()
      }
    }

    let new_state = match &mut self.current_state {
      Lifecycle::Initializing => {
        self.render_thread.run();
        Lifecycle::Start
      }
      Lifecycle::Start => {
        let message = poll_window_or_wait(self, should_wait);
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
      Lifecycle::EndFrame { message } => {
        let message = mem::take(message);
        match self.render_or_exit(&message) {
          Ok(value) => {
            if value {
              Lifecycle::Exiting
            } else {
              let message = poll_window_or_wait(self, should_wait);
              if let Some(message) = message {
                Lifecycle::BeginFrame { message }
              } else {
                Lifecycle::Exiting
              }
            }
          }
          Err(error) => {
            error!("{error}");
            Lifecycle::Exiting
          }
        }
      }
      Lifecycle::Exiting => Lifecycle::ExitLoop,
      Lifecycle::ExitLoop => return None,
    };

    self.current_state = new_state;

    // debug!("{:?}", self.current_state);
    Some(&self.current_state)
  }

  fn render_or_exit(&mut self, received_message: &WindowMessage) -> anyhow::Result<bool> {
    match received_message {
      WindowMessage::Closed => {
        self.game_mailbox.send(GameLoopMessage::Exit)?;
        self.sync_barrier.wait();

        self.render_thread.join();
        Ok(true)
      }
      _ => {
        self.game_mailbox.send(GameLoopMessage::RenderData(RenderData {}))?;
        self.sync_barrier.wait();
        
        Ok(false)
      }
    }
  }
}
