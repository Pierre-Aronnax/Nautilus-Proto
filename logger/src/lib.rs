// lib.rs

pub mod config;
pub mod logger;
pub use tracing;
pub use config::{LoggingConfig, GeneralConfig, FilePathsConfig};
pub use logger::initialize_logger;

// Include macros from macros.rs
#[macro_use]
mod macros;

pub fn setup_logger(config_path: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let config = match config_path {
        Some(path) => LoggingConfig::from_file(path)?,
        None => LoggingConfig::default(),
    };

    initialize_logger(&config)?;
    Ok(())
}
