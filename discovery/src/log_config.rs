//log_config.rs

#[cfg(feature = "logging")]
pub fn setup_logging() {
    logger::init_logging!();
}