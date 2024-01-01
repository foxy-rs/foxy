use strum::Display;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

#[allow(unused)]
#[derive(Default, Display)]
#[strum(serialize_all = "snake_case")]
pub enum Level {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

impl From<Level> for LevelFilter {
    fn from(value: Level) -> Self {
        match value {
            Level::Trace => LevelFilter::TRACE,
            Level::Debug => LevelFilter::DEBUG,
            Level::Info => LevelFilter::INFO,
            Level::Warn => LevelFilter::WARN,
            Level::Error => LevelFilter::ERROR,
        }
    }
}

#[allow(unused)]
pub fn init_max_debug() {
    init_debug(Some(Level::Trace));
}

#[allow(unused)]
pub fn init_debug(_user_logging_level: Option<Level>) {
    #[cfg(debug_assertions)]
    init(_user_logging_level);
}

#[allow(unused)]
pub fn init_max() {
    init(Some(Level::Trace));
}

#[allow(unused)]
pub fn init(user_logging_level: Option<Level>) {
    const NAME: &str = env!("CARGO_PKG_NAME");
    let filter = format!(
        "RUST_LOG=off,{NAME}={}",
        match &user_logging_level {
            None => "off".to_string(),
            Some(level) => level.to_string(),
        }
    );
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_thread_names(true)
        .init();
}

#[allow(unused)]
pub fn lib_info() {
    const NAME: &str = env!("CARGO_PKG_NAME");
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    // const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

    info!("{NAME} v{VERSION}");
    // info!("AUTHORS: [{AUTHORS}]");
}
