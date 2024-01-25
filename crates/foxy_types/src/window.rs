#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum Visibility {
  Shown,
  Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum ColorMode {
  Dark,
  Light,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(unused)]
pub enum CloseBehavior {
  Default,
  Custom,
}
