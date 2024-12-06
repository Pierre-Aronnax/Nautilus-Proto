
use discovery::{Discovery,SSDPService};
use tokio::signal;

#[tokio::main]
async fn main() {
    let mut discovery = Discovery::new();


    // Configure SSDP service
    let ssdp_service = SSDPService::new(
        "urn:schemas-upnp-org:service:ExampleService:1",
        "http://example.local:8080/device-description.xml",
        "uuid:example-service-1",
    );
    discovery.configure_ssdp(ssdp_service).await;

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