use crate::log::level::LogLevel;

#[allow(unused)]
pub fn init_slice(crate_logging_levels: &[(&str, Option<LogLevel>)]) {
  // const NAME: &str = env!("CARGO_PKG_NAME");
  let mut filter = String::from("RUST_LOG=off,");
  let joined: String = crate_logging_levels
    .iter()
    .map(|(name, level)| {
      format!("{name}={}", match &level {
        None => "off".to_string(),
        Some(level) => level.to_string(),
      })
    })
    .collect::<Vec<String>>()
    .join(",");
  filter.push_str(&joined);

  tracing_subscriber::fmt()
    .with_env_filter(filter)
    .with_thread_names(true)
    // .with_file(true)
    // .with_line_number(true)
    .init();
}

#[macro_export]
macro_rules! log_init_max {
  () => {{
    const NAME: &str = env!("CARGO_PKG_NAME");
    $crate::log::init::init_slice(&[("RUST_LOG", None), (NAME, Some($crate::log::level::LogLevel::Trace))])
  }};
}

#[macro_export]
macro_rules! log_init {
  ($level:expr) => {{
    const NAME: &str = env!("CARGO_PKG_NAME");
    $crate::log::init::init_slice(&[("RUST_LOG", None), (NAME, $level)])
  }};
}

/// example: `ezwin::init_with_levels!(("ookami", Some(log::Level::Trace)), ("ezwin", None));`
#[macro_export]
macro_rules! log_init_multiple {
    ($($levels:expr),+) => {{
        $crate::log::init::init_slice(&[("RUST_LOG", None), $($levels),+])
    }};
}

#[macro_export]
macro_rules! log_init_with_others {
    ($level:expr, $($levels:expr),+) => {{
        const NAME: &str = env!("CARGO_PKG_NAME");
        $crate::log::init::init_slice(&[("RUST_LOG", None), (NAME, $level), $($levels),+])
    }};
}
