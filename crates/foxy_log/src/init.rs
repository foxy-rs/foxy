use crate::format::format_filter_slice;
use crate::level::LogLevel;

#[allow(unused)]
pub fn init_slice(crate_logging_levels: &[(&str, Option<LogLevel>)]) {
  let mut filter = format_filter_slice(crate_logging_levels);

  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_thread_names(true)
    .init();
}

#[macro_export]
macro_rules! log_init_max {
  () => {{
    const NAME: &str = env!("CARGO_PKG_NAME");
    $crate::init::init_slice(&[("RUST_LOG", None), (NAME, Some($crate::level::LogLevel::Trace))])
  }};
}

#[macro_export]
macro_rules! log_init {
  ($level:expr) => {{
    const NAME: &str = env!("CARGO_PKG_NAME");
    $crate::init::init_slice(&[("RUST_LOG", None), (NAME, $level)])
  }};
}

/// example: `ezwin::init_with_levels!(("ookami", Some(log::Level::Trace)),
/// ("ezwin", None));`
#[macro_export]
macro_rules! log_init_multiple {
    ($($levels:expr),+) => {{
        $crate::init::init_slice(&[("RUST_LOG", None), $($levels),+])
    }};
}

#[macro_export]
macro_rules! log_init_with_others {
    ($level:expr, $($levels:expr),+) => {{
        const NAME: &str = env!("CARGO_PKG_NAME");
        $crate::init::init_slice(&[("RUST_LOG", None), (NAME, $level), $($levels),+])
    }};
}
