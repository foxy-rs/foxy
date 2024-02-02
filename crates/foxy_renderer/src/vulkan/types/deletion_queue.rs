use std::collections::VecDeque;

#[derive(Default)]
pub struct DeletionQueue {
  deletors: VecDeque<Box<dyn FnOnce() + 'static + Send>>,
}

impl DeletionQueue {
  pub fn new() -> Self {
    Self {
      deletors: Default::default(),
    }
  }

  pub fn push<F: FnOnce() + 'static + Send>(&mut self, f: F) {
    self.deletors.push_back(Box::new(f));
  }

  pub fn flush(&mut self) {
    while let Some(deletor) = self.deletors.pop_back() {
      deletor();
    }
  }
}
