use tracing_subscriber::fmt::SubscriberBuilder;

pub mod builder;
pub mod format;
pub mod init;
pub mod level;
pub mod prelude;

#[allow(unused)]
pub fn session() -> SubscriberBuilder {
    tracing_subscriber::fmt()
}

#[macro_export]
macro_rules! log_lib_info {
    () => {{
        const NAME: &str = env!("CARGO_PKG_NAME");
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        // const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

        tracing::info!("{NAME} v{VERSION}");
    }};
}
