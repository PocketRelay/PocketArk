use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Log file name
pub const LOG_FILE_NAME: &str = "data/server.log";

/// Setup function for setting up the Log4rs logging configuring it
/// for all the different modules and and setting up file and stdout logging
pub async fn setup(logging_level: LevelFilter) {
    if logging_level == LevelFilter::OFF {
        return;
    }

    if let Some(parent) = std::path::Path::new(LOG_FILE_NAME).parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new(format!(
            "debug,sqlx={logging_level},pocket_ark={logging_level}",
        ))
    });

    let stdout_layer = fmt::layer().with_target(true).with_ansi(true);

    let file_appender = tracing_appender::rolling::never("data", "server.log");
    let (non_blocking_file, _guard) = tracing_appender::non_blocking(file_appender);
    Box::leak(Box::new(_guard));

    let file_layer = fmt::layer()
        .with_writer(non_blocking_file)
        .with_target(true)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    log_panics::init();
}

#[cfg(test)]
pub fn setup_test_logging() {
    let filter = EnvFilter::new("debug");

    let stdout_layer = fmt::layer().with_target(true).with_ansi(true);

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .try_init();

    log_panics::init();
}
