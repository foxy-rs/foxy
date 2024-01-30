use std::thread::JoinHandle;

use tracing::*;

pub struct EngineThread<Loop: ThreadLoop> {
  join_handle: Option<JoinHandle<anyhow::Result<()>>>,
  thread_loop: Option<Loop>,
}

impl<Loop: ThreadLoop> EngineThread<Loop> {
  pub fn new(thread_loop: Loop) -> Self {
    Self {
      join_handle: None,
      thread_loop: Some(thread_loop),
    }
  }

  pub fn run(&mut self, info: Loop::Params) {
    if let Some(thread_loop) = self.thread_loop.take() {
      self.join_handle = thread_loop.run(info).inspect_err(|e| error!("{e}")).ok();
    }
  }

  pub fn join(&mut self) {
    if let Some(join_handle) = self.join_handle.take() {
      if let Err(error) = join_handle.join() {
        error!("{error:?}");
      } else {
        trace!("{} thread joined sucessfully", Loop::THREAD_ID);
      }
    } else {
      error!("{} thread join_handle was None!", Loop::THREAD_ID);
    }
  }
}

pub trait ThreadLoop {
  const THREAD_ID: &'static str;
  type Params;

  fn run(self, info: Self::Params) -> anyhow::Result<JoinHandle<anyhow::Result<()>>>
  where
    Self: Sized;
}
