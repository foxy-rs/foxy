use crate::log::prelude::LogLevel;

#[allow(unused)]
pub fn format_filter_slice(crate_logging_levels: &[(&str, Option<LogLevel>)]) -> String {
  // const NAME: &str = env!("CARGO_PKG_NAME");
  let mut filter = String::from("RUST_LOG=off,");
  let joined: String = crate_logging_levels
    .iter()
    .map(|(name, level)| {
      format!(
        "{name}={}",
        match &level {
          None => "off".to_string(),
          Some(level) => level.to_string(),
        }
      )
    })
    .collect::<Vec<String>>()
    .join(",");
  filter.push_str(&joined);

  filter
}

#[macro_export]
macro_rules! log_filter_max {
  () => {{
    const NAME: &str = env!("CARGO_PKG_NAME");
    $crate::log::format::format_filter_slice(&[("RUST_LOG", None), (NAME, Some($crate::log::level::LogLevel::Trace))])
  }};
}

#[macro_export]
macro_rules! log_filter {
  ($level:expr) => {{
    const NAME: &str = env!("CARGO_PKG_NAME");
    $crate::log::format::format_filter_slice(&[("RUST_LOG", None), (NAME, $level)])
  }};
}

/// example: `ezwin::init_with_levels!(("ookami", Some(log::Level::Trace)), ("ezwin", None));`
#[macro_export]
macro_rules! log_filter_multiple {
    ($($levels:expr),+) => {{
        $crate::log::format::format_filter_slice(&[("RUST_LOG", None), $($levels),+])
    }};
}

/// this is different from `log_filter_multiple` in that it also has a dedicated first argument for the current crate.
#[macro_export]
macro_rules! log_filter_with_others {
    ($level:expr, $($levels:expr),+) => {{
        const NAME: &str = env!("CARGO_PKG_NAME");
        $crate::log::format::format_filter_slice(&[("RUST_LOG", None), (NAME, $level), $($levels),+])
    }};
}
