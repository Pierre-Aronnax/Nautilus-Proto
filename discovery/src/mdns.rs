// mdns.rs
// mdns main module

//? Contains the main MDNS Public Structure for access



mod packet;
mod record;
mod name;


use name::DnsName;
use record::DnsRecord;
use packet::DnsPacket;

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::Duration;

#[derive(Clone)]
pub struct MDNSService {
    pub service_name: DnsName,
    pub service_type: DnsName,
    pub hostname: DnsName,
    port: u16,
    txt_data: Option<Vec<u8>>,
}

impl MDNSService {
    pub fn new(service_name: &str, service_type: &str, hostname: &str, port: u16, txt_data: Option<Vec<u8>>) -> Self {
        Self {
            service_name: DnsName::new(service_name).expect("Invalid service name"),
            service_type: DnsName::new(service_type).expect("Invalid service type"),
            hostname: DnsName::new(hostname).expect("Invalid hostname"),
            port,
            txt_data,
        }
    }
}

pub struct MDNSResponder {
    service: MDNSService,
    socket: UdpSocket,
}

impl MDNSResponder {
    /// Creates a new mDNS responder
    pub async fn new(service: MDNSService) -> Self {
        let socket = UdpSocket::bind("0.0.0.0:5454")
            .await
            .expect("Failed to bind to mDNS port");
    
        // Join the mDNS multicast group (224.0.0.251)
        let multicast_addr: Ipv4Addr = "224.0.0.251".parse().unwrap();
        let interface_addr: Ipv4Addr = "0.0.0.0".parse().unwrap();
    
        socket
            .join_multicast_v4(multicast_addr, interface_addr)
            .expect("Failed to join multicast group");
    
        socket
            .set_multicast_loop_v4(true)
            .expect("Failed to enable multicast loop");
    
        Self { service, socket }
    }
    pub async fn run(&mut self, running: Arc<RwLock<bool>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let multicast_addr: SocketAddrV4 = "224.0.0.251:5454".parse().unwrap();
        let mut buffer = [0u8; 4096];
    
        println!("mDNS Responder is running.");
    
        while *running.read().await {
            tokio::select! {
                result = self.socket.recv_from(&mut buffer) => {
                    match result {
                        Ok((size, src)) => {
                            let request_packet = &buffer[..size];
                            println!("Received mDNS packet from {}", src);
    
                            // Parse and handle the query
                            if let Ok(parsed_packet) = DnsPacket::parse(request_packet) {
                                println!("Parsed mDNS packet: {:?}", parsed_packet);
                                self.handle_query(&parsed_packet, src).await?;
                            } else {
                                eprintln!("Failed to parse mDNS packet");
                            }
                        }
                        Err(e) => eprintln!("Error receiving packet: {}", e),
                    }
                }
    
                _ = tokio::time::sleep(Duration::from_secs(10)) => {
                    let announcement_packet = self.create_announcement_packet();
                    if let Err(e) = self.socket.send_to(&announcement_packet, multicast_addr).await {
                        eprintln!("Failed to send announcement: {}", e);
                    } else {
                        println!("Sent announcement to multicast group.");
                    }
                }
            }
        }
    
        println!("mDNS Responder is stopping.");
        Ok(())
    }

    /// Handle an incoming mDNS query
    async fn handle_query(&self, packet: &DnsPacket, src: SocketAddr) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "logging")]
        logger::log_event!(
            INFO,
            "mDNS Event",
            message = format!("Peer Found: {}", src),
            addr = format!("{}",src)
        );
        // Check each question in the query
        for question in &packet.questions {
            println!("Checking question: {:?}", question);
    
            // Match the question's name with the service type
            if question.qname == self.service.service_type {
                println!("Query matches our service type: {}", question.qname);
    
                // Respond with our service information
                let response_packet = self.create_response_packet();
                self.socket.send_to(&response_packet, src).await?;
                println!("Sent mDNS response to {}", src);
            } else {
                println!("Query does not match our service type: {}", question.qname);
            }
        }
        Ok(())
    }

    fn get_local_ip(&self) -> Option<Ipv4Addr> {
        // Use std::net::UdpSocket (synchronous version)
        let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?; // Bind to any available port
        socket.connect("8.8.8.8:53").ok()?; // Connect to a public server to determine local IP
    
        // Retrieve the local address
        if let Ok(local_addr) = socket.local_addr() {
            if let std::net::SocketAddr::V4(v4_addr) = local_addr {
                return Some(*v4_addr.ip());
            }
        }
        None // Return None if no valid IP is found
    }

    fn create_announcement_packet(&self) -> Vec<u8> {
        let mut packet = DnsPacket::new();

        // Add PTR record
        packet.answers.push(DnsRecord::PTR {
            name: self.service.service_type.clone(),
            ttl: 120,
            ptr_name: self.service.service_name.clone(),
        });

        // Add SRV record
        packet.answers.push(DnsRecord::SRV {
            name: self.service.service_name.clone(),
            ttl: 120,
            priority: 0,
            weight: 0,
            port: self.service.port,
            target: self.service.hostname.clone(),
        });

        // Add TXT record
        packet.answers.push(DnsRecord::TXT {
            name: self.service.service_name.clone(),
            ttl: 120,
            txt_data: self.service.txt_data.clone().unwrap_or_else(|| vec![0x00]),
        });

        // Add A record
        let local_ip = self.get_local_ip().unwrap_or_else(|| {
            eprintln!("Warning: Could not determine local IP. Falling back to 127.0.0.1.");
            Ipv4Addr::new(127, 0, 0, 1)
        });
        packet.additionals.push(DnsRecord::A {
            name: self.service.hostname.clone(),
            ttl: 120,
            ip: local_ip.octets(),
        });

        packet.serialize()
    }

    fn create_response_packet(&self) -> Vec<u8> {
        // TODO: Implement the response packet logic
        vec![]
    }
}


