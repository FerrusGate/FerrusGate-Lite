use crate::config::LogConfig;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;

pub fn init_logging(config: &LogConfig) -> WorkerGuard {
    // Create writer based on config
    let writer: Box<dyn std::io::Write + Send + Sync> = if let Some(ref log_file) = config.file {
        if !log_file.is_empty() && config.enable_rotation {
            // Use rolling log files
            let dir = std::path::Path::new(log_file)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let filename = std::path::Path::new(log_file)
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("shortlinker.log"));
            let filename_str = filename.to_str().unwrap_or("shortlinker.log");
            let appender = rolling::Builder::new()
                .rotation(rolling::Rotation::DAILY)
                .filename_prefix(filename_str.trim_end_matches(".log"))
                .filename_suffix("log")
                .max_log_files(config.max_backups as usize)
                .build(dir)
                .expect("Failed to create rolling log appender");
            Box::new(appender)
        } else if !log_file.is_empty() {
            // Non-rotating, append to file
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
                .expect("Failed to open log file");
            Box::new(file)
        } else {
            // Empty filename, output to console
            Box::new(std::io::stdout())
        }
    } else {
        // Output to console
        Box::new(std::io::stdout())
    };

    let (non_blocking_writer, guard) = tracing_appender::non_blocking(writer);
    let filter = tracing_subscriber::EnvFilter::new(config.level.clone());

    let subscriber_builder = tracing_subscriber::fmt()
        .with_writer(non_blocking_writer)
        .with_env_filter(filter)
        .with_level(true)
        .with_ansi(config.file.as_ref().is_none_or(|f| f.is_empty()));

    if config.format == "json" {
        subscriber_builder.json().init();
    } else {
        subscriber_builder.init();
    }

    tracing::info!("Logging initialized with level: {}", config.level);

    guard
}
