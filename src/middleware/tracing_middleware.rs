use tracing::info;
use tracing_subscriber::{filter, prelude::*, EnvFilter};

pub async fn init_subscriber() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new(tracing::Level::DEBUG.as_str()));
    let debug_logs_file_appender = tracing_appender::rolling::hourly("logs", "debug_logs.log");

    let debug_log = tracing_subscriber::fmt::layer();
    let stdout_log = tracing_subscriber::fmt::layer().pretty();

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_log)
        .with(
            debug_log
                .with_writer(debug_logs_file_appender)
                .with_ansi(false)
                .with_filter(filter::LevelFilter::DEBUG),
        );

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to install `tracing` subscriber.");
    info!("Tracing subscriber inited")
}
