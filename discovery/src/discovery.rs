// discovery.rs

use crate::mdns::{MDNSResponder, MDNSService};
use crate::ssdp::{SSDPResponder,SSDPService};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

#[cfg(feature = "logging")]
use crate::log_config::setup_logging;

pub struct Discovery {
    mdns: Option<Arc<RwLock<MDNSResponder>>>, // Optional mDNS responder instance
    ssdp: Option<Arc<RwLock<SSDPResponder>>>, // Optional SSDP responder instance
    running: Arc<RwLock<bool>>,               // Shared running state
    tasks: Vec<JoinHandle<()>>,               // Background tasks
}

impl Discovery {
    /// Create a new Discovery instance
    pub fn new() -> Self {
        #[cfg(feature = "logging")]
        setup_logging();
    
        #[cfg(not(feature = "logging"))]
        println!("Logging is disabled. Enable the 'logging' feature to activate it.");
    
        Self {
            mdns: None,
            ssdp: None,
            running: Arc::new(RwLock::new(false)),
            tasks: Vec::new(),
        }
    }

    /// Configure mDNS protocol (async)
    pub async fn configure_mdns(&mut self, service: MDNSService) {
        self.mdns = Some(Arc::new(RwLock::new(MDNSResponder::new(service).await)));
    }

    pub async fn configure_ssdp(&mut self, service: SSDPService) {
        self.ssdp = Some(Arc::new(RwLock::new(
            SSDPResponder::new(service)
                .await
                .expect("Failed to create SSDP responder"),
        )));
    }

    /// Start all configured discovery protocols
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut running = self.running.write().await;
            *running = true;
        }

        // Start mDNS if configured
        if let Some(mdns) = &self.mdns {
            let mdns_instance = Arc::clone(mdns);
            let running = Arc::clone(&self.running);

            let task = tokio::spawn(async move {
                let mut responder = mdns_instance.write().await;
                if let Err(e) = responder.run(running).await {
                    eprintln!("mDNS error: {}", e);
                }
            });

            self.tasks.push(task);
        }

        if let Some(ssdp) = &self.ssdp {
            let ssdp_instance = Arc::clone(ssdp);
            let running = Arc::clone(&self.running);

            let task = tokio::spawn(async move {
                let mut responder = ssdp_instance.write().await;
                if let Err(e) = responder.run(running).await {
                    eprintln!("SSDP error: {}", e);
                }
            });

            self.tasks.push(task);
        }

        println!("Discovery protocols started.");
        Ok(())
    }

    /// Stop all configured discovery protocols
    pub async fn stop(&mut self) {
        {
            let mut running = self.running.write().await;
            *running = false;
        }

        // Wait for all tasks to finish
        for task in self.tasks.drain(..) {
            if let Err(e) = task.await {
                eprintln!("Error stopping task: {:?}", e);
            }
        }

        println!("Discovery protocols stopped.");
    }
}
