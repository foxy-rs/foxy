use std::{
  sync::{Arc, Barrier},
  thread::JoinHandle,
  time::Duration,
};

use crossbeam::{
  channel::{Receiver, Sender, TryRecvError, TrySendError},
  queue::ArrayQueue,
};
use egui::RawInput;
use foxy_renderer::renderer::{render_data::RenderData, Renderer};
use foxy_time::{timer::Timer, Time, TimeSettings};
use tracing::*;
use witer::prelude::*;

use super::{
  builder::{DebugInfo, FoxySettings},
  runnable::Runnable,
  state::Foxy,
  FoxyResult,
};
use crate::{
  core::{
    message::{GameLoopMessage, RenderLoopMessage},
    runnable::Flow,
  },
  foxy_error,
};

pub struct Framework {
  window: Arc<Window>,
  preferred_visibility: Visibility,
  message_sender: Sender<Message>,
  sync_barrier: Arc<Barrier>,

  // render_queue: Arc<ArrayQueue<RenderData>>,
  // render_mailbox: Mailbox<RenderLoopMessage, GameLoopMessage>,
  game_thread: JoinHandle<FoxyResult<()>>,
}

impl Framework {
  pub fn new<App: Runnable>(mut settings: FoxySettings) -> FoxyResult<Self> {
    let preferred_visibility = settings.window.visibility;
    settings.window.visibility = Visibility::Hidden;
    settings.window.close_on_x = false;

    let window = Arc::new(Window::new(settings.window.clone())?);

    Self::initialize::<App>(settings, window, preferred_visibility)
  }
}

impl Framework {
  const GAME_THREAD_ID: &'static str = "foxy";
  const MAX_FRAME_DATA_IN_FLIGHT: usize = 2;

  fn initialize<App: Runnable>(
    settings: FoxySettings,
    window: Arc<Window>,
    preferred_visibility: Visibility,
  ) -> FoxyResult<Self> {
    trace!("Firing up Foxy");

    let (message_sender, message_receiver) = crossbeam::channel::unbounded();

    let sync_barrier = Arc::new(Barrier::new(2));

    let game_thread = Self::game_loop::<App>(
      window.clone(),
      settings.time,
      settings.debug_info,
      sync_barrier.clone(),
      message_receiver,
    )?;

    Ok(Self {
      window,
      preferred_visibility,
      message_sender,
      sync_barrier,
      game_thread,
    })
  }

  pub fn run(self) -> FoxyResult<()> {
    info!("KON KON KITSUNE!");

    self.sync_barrier.wait();

    debug!("Kicking off window loop");

    let window = self.window.clone();
    for message in window.as_ref() {
      let should_sync = matches!(message, Message::Window(WindowMessage::Resized(..)));

      if message.is_some() {
        self.message_sender.try_send(message).unwrap();
      }

      if should_sync {
        self.sync_barrier.wait();
      }
    }

    debug!("Wrapping up window loop");

    self.game_thread.join().map_err(|e| foxy_error!("{e:?}"))??;
    // self.renderer.delete();

    info!("OTSU KON DESHITA!");

    Ok(())
  }

  fn game_loop<App: Runnable>(
    window: Arc<Window>,
    time_settings: TimeSettings,
    debug_info: DebugInfo,
    sync_barrier: Arc<Barrier>,
    message_receiver: Receiver<Message>,
  ) -> FoxyResult<JoinHandle<FoxyResult<()>>> {
    let handle = std::thread::Builder::new()
      .name(Self::GAME_THREAD_ID.into())
      .spawn(move || -> FoxyResult<()> {
        let mut foxy = Foxy::new(window, time_settings, debug_info)?;

        sync_barrier.wait();

        debug!("Kicking off game loop");

        let mut app = App::new(&mut foxy);
        app.start(&mut foxy);

        loop {
          let message = message_receiver.try_recv().ok().unwrap_or_default();

          let raw_input: RawInput = foxy.take_egui_raw_input();

          match &message {
            Message::Window(WindowMessage::Resized(..)) => {
              foxy.renderer.resize();
              sync_barrier.wait();
            }
            Message::Window(WindowMessage::CloseRequested) if app.stop(&mut foxy) == Flow::Exit => {
              foxy.window.close();
              continue;
            }
            Message::ExitLoop => break,
            _ => (),
          }

          foxy.time.update();
          while foxy.time.should_do_tick_unchecked() {
            foxy.time.tick();
            app.fixed_update(&mut foxy, &message);
          }
          app.update(&mut foxy, &message);

          let _full_output = foxy.egui_context.run(raw_input, |ui| {
            app.egui(&foxy, ui);
          });

          if !foxy.render() {
            foxy.window.close();
          }
        }

        trace!("Exiting game!");

        app.delete();
        foxy.renderer.delete();

        debug!("Wrapping up game loop");
        Ok(())
      })?;

    Ok(handle)
  }
}
