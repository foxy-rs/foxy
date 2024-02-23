use tracing_subscriber::{
  fmt::{
    format::{DefaultFields, Format},
    SubscriberBuilder,
  },
  EnvFilter,
};

pub struct LoggingSession {
  filter: EnvFilter,
  thread_names: bool,
  file_names: bool,
  line_numbers: bool,
}

impl Default for LoggingSession {
  fn default() -> Self {
    Self::new()
  }
}

impl LoggingSession {
  pub fn new() -> Self {
    Self {
      filter: String::new().into(),
      thread_names: true,
      file_names: false,
      line_numbers: false,
    }
  }

  pub fn with_filter(self, filter: impl Into<EnvFilter>) -> Self {
    Self {
      filter: filter.into(),
      thread_names: self.thread_names,
      file_names: self.file_names,
      line_numbers: self.line_numbers,
    }
  }

  pub fn with_thread_names(self, enable: bool) -> Self {
    Self {
      filter: self.filter,
      thread_names: enable,
      file_names: self.file_names,
      line_numbers: self.line_numbers,
    }
  }

  pub fn with_file_names(self, enable: bool) -> Self {
    Self {
      filter: self.filter,
      thread_names: self.thread_names,
      file_names: enable,
      line_numbers: self.line_numbers,
    }
  }

  pub fn with_line_numbers(self, enable: bool) -> Self {
    Self {
      filter: self.filter,
      thread_names: self.thread_names,
      file_names: self.file_names,
      line_numbers: enable,
    }
  }

  pub fn finalize(self) -> SubscriberBuilder<DefaultFields, Format, EnvFilter> {
    tracing_subscriber::fmt()
      .with_env_filter(self.filter)
      .with_thread_names(self.thread_names)
      .with_file(self.file_names)
      .with_line_number(self.line_numbers)
  }

  pub fn start(self) {
    tracing_subscriber::fmt()
      .with_env_filter(self.filter)
      .with_thread_names(self.thread_names)
      .with_file(self.file_names)
      .with_line_number(self.line_numbers)
      .init();
  }
}

#[macro_export]
macro_rules! logging_session {
  () => {{
    const NAME: &str = env!("CARGO_PKG_NAME");
    $crate::log::builder::LoggingSession::new().with_filter($crate::log::format::format_filter_slice(&[
      ("RUST_LOG", None),
      (NAME, Some($crate::log::level::LogLevel::Trace)),
    ]))
  }};
}

#[macro_export]
macro_rules! logging_session_ex {
    ($($levels:expr),+) => {{
        const NAME: &str = env!("CARGO_PKG_NAME");
        $crate::log::builder::LoggingSession::new()
            .with_filter($crate::log::format::format_filter_slice(&[("RUST_LOG", None), (NAME, Some($crate::log::level::LogLevel::Trace)), $($levels),+]))
    }};
}

#[macro_export]
macro_rules! debug_logging_session {
  () => {{
    if cfg!(debug_assertions) {
      Some($crate::logging_session!())
    } else {
      None
    }
  }};
}

#[macro_export]
macro_rules! debug_logging_session_ex {
  ($($levels:expr),+) => {{
    if cfg!(debug_assertions) {
      Some($crate::logging_session_ex!($($levels),+))
    } else {
      None
    }
  }};
}

#[macro_export]
macro_rules! start_debug_logging_session {
  () => {{
    #[cfg(debug_assertions)]
    $crate::logging_session!().start();
  }};
}

#[macro_export]
macro_rules! start_debug_logging_session_ex {
  ($($levels:expr),+) => {{
    #[cfg(debug_assertions)]
    $crate::logging_session_ex!($($levels),+).start();
  }};
}
