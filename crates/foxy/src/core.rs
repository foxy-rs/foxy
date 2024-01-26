use self::{
  builder::{FoxyBuilder, FoxyCreateInfo, HasSize, HasTitle, MissingSize, MissingTitle},
  lifecycle::Stage,
  render_loop::RenderLoop,
  time::Time,
};
use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_types::{thread::EngineThread, window::Polling};
use foxy_util::log::LogErr;
use foxy_window::prelude::*;
use message::{GameLoopMessage, RenderLoopMessage};
use messaging::Mailbox;
use std::{
  marker::PhantomData,
  mem,
  sync::{Arc, Barrier},
};
use tracing::*;

pub mod builder;
pub mod lifecycle;
mod message;
mod render_loop;
pub mod time;

pub struct FoxyFramework {
  pub time: Time,
  pub window: Window,
}

pub struct Foxy<'a> {
  polling_strategy: Polling,
  render_thread: EngineThread<RenderLoop>,
  game_mailbox: Mailbox<GameLoopMessage, RenderLoopMessage>,
  sync_barrier: Arc<Barrier>,

  current_stage: Option<Stage>,
  foxy: Option<FoxyFramework>,
  _phantom: PhantomData<&'a ()>,
}

impl Foxy<'_> {
  pub fn builder() -> FoxyBuilder<MissingTitle, MissingSize> {
    Default::default()
  }

  pub(crate) fn new(create_info: FoxyCreateInfo<HasTitle, HasSize>) -> anyhow::Result<Self> {
    trace!("Firing up Foxy");

    let time = Time::new(128.0, 1024);

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

    let current_stage = Some(Stage::Initializing);

    Ok(Self {
      current_stage,
      render_thread,
      game_mailbox,
      sync_barrier,
      polling_strategy: create_info.polling_strategy,
      foxy: Some(FoxyFramework { time, window }),
      _phantom: PhantomData,
    })
  }

  // pub fn time(&self) -> &Time {
  //   &self.time
  // }

  // pub fn window(&self) -> &Window {
  //   &self.window
  // }

  // pub fn poll(&mut self) -> Option<Lifecycle> {
  //   self.next_state(false)
  // }

  // pub fn wait(&mut self) -> Option<Lifecycle> {
  //   self.next_state(true)
  // }

  fn next_window_message(&mut self, window: &mut Window) -> Option<WindowMessage> {
    if let Polling::Wait = self.polling_strategy {
      window.wait()
    } else {
      window.next()
    }
  }

  fn next_state(&mut self) -> Option<&Stage> {
    let old_stage = self.current_stage.take().expect("stage cannot be None");
    let new_state = match old_stage {
      Stage::Initializing => {
        self.render_thread.run(());
        Stage::Start { foxy: self.foxy.take().expect("foxy cannot be None") }
      }
      Stage::Start { mut foxy } => {
        info!("KON KON KITSUNE!");
        let message = self.next_window_message(&mut foxy.window);
        if let Some(message) = message {
          Stage::BeginFrame { foxy, message }
        } else {
          Stage::Exiting { foxy }
        }
      }
      Stage::BeginFrame { foxy, message } => {
        self.sync_barrier.wait();

        Stage::EarlyUpdate { foxy, message }
      }
      Stage::EarlyUpdate { mut foxy, message } => {
        foxy.time.update();
        if foxy.time.should_do_tick() {
          foxy.time.tick();
          Stage::FixedUpdate { foxy, message }
        } else {
          Stage::Update { foxy, message }
        }
      }
      Stage::FixedUpdate { mut foxy, message } => {
        if foxy.time.should_do_tick() {
          foxy.time.tick();
          Stage::FixedUpdate { foxy, message }
        } else {
          Stage::Update { foxy, message }
        }
      }
      Stage::Update { foxy, message } => {
        Stage::EndFrame { foxy, message }
      }
      Stage::EndFrame { mut foxy, .. } => {
        let _ = self
          .game_mailbox
          .send(GameLoopMessage::RenderData(RenderData {}))
          .log_error();
        self.sync_barrier.wait();

        let message = self.next_window_message(&mut foxy.window);
        if let Some(message) = message {
          Stage::BeginFrame { foxy, message }
        } else {
          Stage::Exiting { foxy }
        }
      }
      Stage::Exiting { foxy } => {
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

    self.current_stage = Some(new_state);

    // debug!("{:?}", self.current_state);
    self.current_stage.as_ref()
  }
}

impl<'a> Iterator for Foxy<'a> {
  type Item = &'a Stage;

  fn next(&mut self) -> Option<Self::Item> {
    // it is irrefutable that the reference not outlive Foxy
    unsafe { std::mem::transmute(self.next_state()) }
  }
}
