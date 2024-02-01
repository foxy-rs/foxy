use std::thread::JoinHandle;

use tracing::*;

use super::error::ThreadError;
use crate::log::LogErr;

pub trait ThreadLoop {
  type Params;

  fn run(self, thread_id: String, params: Self::Params) -> Result<JoinHandle<Result<(), ThreadError>>, ThreadError>
  where
    Self: Sized;

  fn _skip() -> bool {
    false
  }
}

pub struct LoopHandle<L: ThreadLoop, A> {
  id: String,

  thread_handle: Option<JoinHandle<Result<(), ThreadError>>>,
  thread_loop: Option<L>,
  thread_args: Option<A>,
}

impl<L: ThreadLoop<Params = A>, A> LoopHandle<L, A> {
  pub fn new(id: &'static str, thread_loop: L, thread_args: A) -> Self {
    Self {
      id: id.into(),
      thread_handle: None,
      thread_loop: Some(thread_loop),
      thread_args: Some(thread_args),
    }
  }

  pub fn run(&mut self) {
    if let Some(thread_loop) = self.thread_loop.take() {
      if let Some(thread_args) = self.thread_args.take() {
        match thread_loop.run(self.id.clone(), thread_args).log_error() {
          Ok(value) => self.thread_handle = Some(value),
          Err(_) => self.thread_handle = None,
        }
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
      error!("`{}` thread handle was None!", self.id);
    }
  }
}
