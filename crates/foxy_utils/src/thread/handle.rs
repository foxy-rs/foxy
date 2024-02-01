use std::thread::JoinHandle;

use tracing::*;

use super::error::ThreadError;
use crate::log::LogErr;

pub type HandlesResult = Result<Vec<JoinHandle<Result<(), ThreadError>>>, ThreadError>;

pub trait ThreadLoop {
  type Params;

  fn run(self, thread_id: Vec<String>, params: Self::Params) -> HandlesResult
  where
    Self: Sized;

  fn _skip() -> bool {
    false
  }
}

pub struct LoopHandle<L: ThreadLoop, A> {
  id: Vec<String>,

  thread_handle: Option<Vec<JoinHandle<Result<(), ThreadError>>>>,
  thread_loop: Option<L>,
  thread_args: Option<A>,
}

impl<L: ThreadLoop<Params = A>, A> LoopHandle<L, A> {
  pub fn new(id: Vec<String>, thread_loop: L, thread_args: A) -> Self {
    Self {
      id,
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
    if let Some(thread_handles) = self.thread_handle.take() {
      for (i, handle) in thread_handles.into_iter().enumerate() {
        if let Err(error) = handle.join() {
          error!("{error:?}");
        } else {
          trace!("`{}` thread has joined.", self.id.get(i).cloned().unwrap_or_default());
        }
      }
    } else {
      error!("`{:?}` thread handle was None!", self.id);
    }
  }
}
