use directories::ProjectDirs;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init() -> Option<WorkerGuard> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let (file_layer, guard) = if let Some(dirs) = ProjectDirs::from("com", "mpjhorner", "IdeUltra")
    {
        let log_dir = dirs.data_dir();
        if std::fs::create_dir_all(log_dir).is_ok() {
            let appender = tracing_appender::rolling::never(log_dir, "log.ndjson");
            let (nb, guard) = tracing_appender::non_blocking(appender);
            let layer = fmt::layer().json().with_writer(nb);
            (Some(layer), Some(guard))
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    let stderr_layer = fmt::layer().with_target(false).compact();

    tracing_subscriber::registry()
        .with(filter)
        .with(stderr_layer)
        .with(file_layer)
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "IdeUltra starting"
    );

    guard
}
