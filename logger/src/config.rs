// /src/config.rs
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub console_output: bool,
}

#[derive(Debug, Deserialize)]
pub struct FilePathsConfig {
    pub info: String,
    pub error: String,
    pub debug: String,
    pub trace: String,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    pub general: GeneralConfig,
    pub files: FilePathsConfig,
}

impl LoggingConfig {
    /// Reads configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&config_content)?;
        config.ensure_directories_exist()?;
        Ok(config)
    }

    /// Creates default configuration and ensures directories exist
    pub fn default() -> Self {
        let config = LoggingConfig {
            general: GeneralConfig {
                console_output: true,
            },
            files: FilePathsConfig {
                info: "./logs/info.log".to_string(),
                error: "./logs/error.log".to_string(),
                debug: "./logs/debug.log".to_string(),
                trace: "./logs/trace.log".to_string(),
            },
        };
        config.ensure_directories_exist().expect("Failed to create log directories");
        config
    }

    /// Ensures that the directories for log files exist
    fn ensure_directories_exist(&self) -> Result<(), Box<dyn std::error::Error>> {
        let log_files = vec![
            &self.files.info,
            &self.files.error,
            &self.files.debug,
            &self.files.trace,
        ];

        for log_file in log_files {
            if let Some(dir) = Path::new(log_file).parent() {
                if !dir.exists() {
                    println!("Creating directory: {:?}", dir); // Debug print
                    fs::create_dir_all(dir)?;
                }
            }
        }

        Ok(())
    }
}

