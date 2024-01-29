#[derive(Default)]
pub struct DeletionQueue {
  deletors: Vec<Box<dyn FnMut()>>,
}

impl DeletionQueue {
  pub fn push(&mut self, func: impl FnMut() + 'static) {
    self.deletors.push(Box::new(func))
  }

  pub fn flush(&mut self) {
    for deleter in &mut self.deletors {
      deleter();
    }
    self.deletors.clear();
  }
}
