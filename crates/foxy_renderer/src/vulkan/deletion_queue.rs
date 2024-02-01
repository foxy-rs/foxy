use std::collections::VecDeque;

#[derive(Default)]
pub struct DeletionQueue {
  deletors: VecDeque<Box<dyn FnMut() + 'static + Send + Sync>>,
}

impl DeletionQueue {
  pub fn new() -> Self {
    Self {
      deletors: Default::default(),
    }
  }

  pub fn push(&mut self, func: impl FnMut() + 'static + Send + Sync) {
    self.deletors.push_back(Box::new(func));
  }

  pub fn flush(&mut self) {
    while let Some(mut deletor) = self.deletors.pop_back() {
      deletor();
    }
  }
}
