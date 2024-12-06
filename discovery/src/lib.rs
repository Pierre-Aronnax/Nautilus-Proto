//lib.rs
mod mdns;
mod ssdp;

// Discovery Servies for Loading
pub use mdns::MDNSService;
pub use ssdp::SSDPService;

// public facing API for Discovery to initialize
mod discovery;
pub use discovery::Discovery;

mod log_config;