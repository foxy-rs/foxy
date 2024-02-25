use std::thread::JoinHandle;

use foxy_log::LogErr;
use tracing::*;

use super::error::ThreadError;

pub type HandlesResult = Result<JoinHandle<Result<(), ThreadError>>, ThreadError>;

pub trait ThreadLoop {
  fn run(self, thread_id: String) -> HandlesResult
  where
    Self: Sized;

  fn _skip() -> bool {
    false
  }
}

pub struct LoopHandle<L: ThreadLoop> {
  id: String,

  thread_handle: Option<JoinHandle<Result<(), ThreadError>>>,
  thread_loop: Option<L>,
}

impl<L: ThreadLoop> LoopHandle<L> {
  pub fn new(id: String, thread_loop: L) -> Self {
    Self {
      id,
      thread_handle: None,
      thread_loop: Some(thread_loop),
    }
  }

  pub fn run(&mut self) {
    if let Some(thread_loop) = self.thread_loop.take() {
      match thread_loop.run(self.id.clone()).log_error() {
        Ok(value) => self.thread_handle = Some(value),
        Err(_) => self.thread_handle = None,
      }
    }
  }

  pub fn join(&mut self) {
    if let Some(thread_handle) = self.thread_handle.take() {
      if let Err(error) = thread_handle.join() {
        error!("{error:?}");
      } else {
        trace!("`{}` thread has joined.", self.id);
      }
    } else {
      error!("`{:?}` thread handle was None!", self.id);
    }
  }
}
