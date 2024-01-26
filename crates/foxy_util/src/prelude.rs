// pub use tracing::{self, trace, debug, info, warn, error};
// pub use tracing;
// pub use tracing_subscriber;

pub use crate::{
  time::{EngineTime, Time}, timer::Timer, stopwatch::Stopwatch,
  log::level::LogLevel, log_filter, log_filter_max, log_filter_multiple, log_filter_with_others, log_init,
  log_init_max, log_init_multiple, log_init_with_others, log_lib_info, logging_session, logging_session_ex,
};

// #[macro_export]
// macro_rules! trace {
//     ($($tts:tt)*) => {
//         tracing::trace!($($tts)*)
//     };
// }

// #[macro_export]
// macro_rules! debug {
//     ($($tts:tt)*) => {
//         tracing::debug!($($tts)*)
//     };
// }

// #[macro_export]
// macro_rules! info {
//     ($($tts:tt)*) => {
//         tracing::info!($($tts)*)
//     };
// }

// #[macro_export]
// macro_rules! warn {
//     ($($tts:tt)*) => {
//         tracing::warn!($($tts)*)
//     };
// }

// #[macro_export]
// macro_rules! error {
//     ($($tts:tt)*) => {
//         tracing::error!($($tts)*)
//     };
// }
