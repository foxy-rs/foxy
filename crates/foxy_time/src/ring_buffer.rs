use std::collections::{vec_deque, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RingBuffer<T> {
  items: VecDeque<T>,
  capacity: usize,
}

#[allow(unused)]
impl<T> RingBuffer<T> {
  pub fn new(capacity: usize) -> Self {
    let mut items = VecDeque::new();
    items.shrink_to(capacity);
    Self { items, capacity }
  }

  pub fn push(&mut self, item: T) -> Option<T> {
    self.items.push_back(item);
    if self.items.len() >= self.capacity {
      self.items.pop_front()
    } else {
      None
    }
  }

  pub fn len(&self) -> usize {
    self.items.len()
  }

  pub fn is_empty(&self) -> bool {
    self.items.len() == 0
  }

  pub fn capacity(&self) -> usize {
    self.capacity
  }

  pub fn iter(&self) -> vec_deque::Iter<T> {
    self.items.iter()
  }

  pub fn iter_mut(&mut self) -> vec_deque::IterMut<T> {
    self.items.iter_mut()
  }
}

impl<T> IntoIterator for RingBuffer<T> {
  type IntoIter = vec_deque::IntoIter<Self::Item>;
  type Item = T;

  fn into_iter(self) -> Self::IntoIter {
    self.items.into_iter()
  }
}
