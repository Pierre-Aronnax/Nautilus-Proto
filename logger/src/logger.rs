// /src/logger.rs
use crate::config::LoggingConfig;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tracing_subscriber::{fmt, layer::SubscriberExt, Layer, Registry};
use tracing_subscriber::fmt::time::UtcTime;
use tracing_subscriber::filter::LevelFilter;
use chrono::Utc;
use std::collections::HashMap;
use tracing::Level;
/// A custom writer to handle size-based log rotation.
struct SizeLimitedAppender {
    path: String,
    max_size: u64,
    current_size: Arc<Mutex<u64>>,
}

impl SizeLimitedAppender {
    fn new(path: String, max_size: u64) -> Self {
        let current_size = Arc::new(Mutex::new(Self::get_file_size(&path).unwrap_or(0)));
        Self {
            path,
            max_size,
            current_size,
        }
    }

    fn get_file_size(path: &str) -> std::io::Result<u64> {
        if let Ok(metadata) = std::fs::metadata(path) {
            Ok(metadata.len())
        } else {
            Ok(0)
        }
    }

    fn rotate_file(&self) -> std::io::Result<()> {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let rotated_path = format!("{}.{}", self.path, timestamp);
        std::fs::rename(&self.path, rotated_path)?;
        *self.current_size.lock().unwrap() = 0; // Reset current size
        Ok(())
    }
}

impl Write for SizeLimitedAppender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut current_size = self.current_size.lock().unwrap();

        if *current_size >= self.max_size {
            self.rotate_file()?;
        }

        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        let bytes_written = file.write(buf)?;
        *current_size += bytes_written as u64;

        Ok(bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut file = OpenOptions::new().create(true).append(true).open(&self.path)?;
        file.flush()
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for SizeLimitedAppender {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        Self {
            path: self.path.clone(),
            max_size: self.max_size,
            current_size: Arc::clone(&self.current_size),
        }
    }
}


/// Generates macros for custom events dynamically
pub fn generate_event_macros(events: &HashMap<String, Level>) {
    for (name, level) in events {
        let macro_name = format!("log_{}", name);
        let macro_code = format!(
            r#"
            #[macro_export]
            macro_rules! {} {{
                ($($arg:tt)*) => {{
                    tracing::event!(tracing::Level::{:?}, $($arg)*);
                }};
            }}
            "#,
            macro_name, level
        );
        println!("Generated macro: {}", macro_code); // Debug output (optional)
    }
}

/// Initializes the logger based on the provided configuration.
pub fn initialize_logger(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Initializing loggers with the following files:");
    println!("Info log: {}", config.files.info);
    println!("Error log: {}", config.files.error);
    println!("Debug log: {}", config.files.debug);
    println!("Trace log: {}", config.files.trace);

    // Create custom size-limited appenders
    let info_appender = SizeLimitedAppender::new(config.files.info.clone(), 100 * 1024 * 1024);
    let error_appender = SizeLimitedAppender::new(config.files.error.clone(), 100 * 1024 * 1024);
    let debug_appender = SizeLimitedAppender::new(config.files.debug.clone(), 100 * 1024 * 1024);
    let trace_appender = SizeLimitedAppender::new(config.files.trace.clone(), 100 * 1024 * 1024);

    // Define a custom time format
    let timer = UtcTime::rfc_3339();

    // Define a log format that includes the timestamp, level, and message
    let log_format = fmt::format()
        .with_timer(timer)
        .with_level(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .compact()
        .with_ansi(false); // Disable ANSI colors

    // Create individual layers for each log level
    let info_layer = fmt::layer()
        .with_writer(info_appender)
        .event_format(log_format.clone())
        .with_filter(LevelFilter::INFO);

    let error_layer = fmt::layer()
        .with_writer(error_appender)
        .event_format(log_format.clone())
        .with_filter(LevelFilter::ERROR);

    let debug_layer = fmt::layer()
        .with_writer(debug_appender)
        .event_format(log_format.clone())
        .with_filter(LevelFilter::DEBUG);

    let trace_layer = fmt::layer()
        .with_writer(trace_appender)
        .event_format(log_format)
        .with_filter(LevelFilter::TRACE);

    let subscriber = Registry::default()
        .with(info_layer)
        .with(error_layer)
        .with(debug_layer)
        .with(trace_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
