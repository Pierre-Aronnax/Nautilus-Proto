
use discovery::{Discovery,MDNSService};
use tokio::signal;

#[tokio::main]
async fn main() {
    let mut discovery = Discovery::new();

    // Configure mDNS service
    let mdns_service = MDNSService::new(
        "ExampleService",
        "_example._tcp",
        "example.local",
        8080,
        Some(b"version=1.0".to_vec()),
    );
    discovery.configure_mdns(mdns_service).await;

    // Start discovery
    discovery.start().await.expect("Failed to start discovery");

    println!("Discovery is running. Press Ctrl+C to stop.");

    // Wait for Ctrl+C to stop the service
    signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C");

    // Stop discovery
    discovery.stop().await;
    println!("Discovery stopped.");
}