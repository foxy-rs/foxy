use strum::Display;
use tracing_subscriber::filter::LevelFilter;

#[allow(unused)]
#[derive(Default, Display, Debug)]
#[strum(serialize_all = "snake_case")]
pub enum LogLevel {
  Trace,
  Debug,
  #[default]
  Info,
  Warn,
  Error,
}

impl From<LogLevel> for LevelFilter {
  fn from(value: LogLevel) -> Self {
    match value {
      LogLevel::Trace => LevelFilter::TRACE,
      LogLevel::Debug => LevelFilter::DEBUG,
      LogLevel::Info => LevelFilter::INFO,
      LogLevel::Warn => LevelFilter::WARN,
      LogLevel::Error => LevelFilter::ERROR,
    }
  }
}
