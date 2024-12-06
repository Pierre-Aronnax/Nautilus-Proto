use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::UdpSocket;

use tokio::time::{self, Duration};

use std::sync::Arc;
use tokio::sync::RwLock;

use chrono; 

#[derive(Clone)]
pub struct SSDPService {
    pub usn: String,        
    pub st: String,      
    pub location: String,   
}

impl SSDPService {
    pub fn new(usn: &str, st: &str, location: &str) -> Self {
        Self {
            usn: usn.to_string(),
            st: st.to_string(),
            location: location.to_string(),
        }
    }
}

pub struct SSDPResponder {
    service: SSDPService,
    socket: UdpSocket,
    announcement_counter: usize,
}

impl SSDPResponder {
    pub async fn new(service: SSDPService) -> Result<Self, Box<dyn std::error::Error>> {
        let socket = UdpSocket::bind("0.0.0.0:1900").await?;
        socket
            .join_multicast_v4(Ipv4Addr::new(239, 255, 255, 250), Ipv4Addr::UNSPECIFIED)
            .expect("Failed to join multicast group");
        Ok(Self { service, socket,announcement_counter: 0, })
    }

    pub async fn run(&mut self, running: Arc<RwLock<bool>>) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = [0u8; 1024];
        let multicast_addr: SocketAddrV4 = "239.255.255.250:1900".parse().unwrap();
        let local_ip = get_local_ip().unwrap_or_else(|| Ipv4Addr::new(127, 0, 0, 1)); // Default to 127.0.0.1 if no IP is found

        while *running.read().await {
            tokio::select! {
                result = self.socket.recv_from(&mut buf) => {
                    match result {
                        Ok((size, src)) => {
                            let request = String::from_utf8_lossy(&buf[..size]);
                            if let std::net::IpAddr::V4(src_ip) = src.ip() {
                                if src_ip == local_ip {
                                    println!("Ignoring self NOTIFY message from {}", src);
                                    continue; 
                                }
                            }
    
                            // Handle NOTIFY messages, print only if the service type matches
                            if request.contains("NOTIFY") {
                                if let Some(nt) = self.extract_nt_from_request(&request) {
                                    if nt == self.service.st {
                                        #[cfg(feature = "logging")]
                                        logger::log_event!(
                                            INFO,
                                            "Peer Discovered",
                                            message = format!("Found a peer: {}", src)
                                        );
                                        println!("Received NOTIFY message from {}: {}", src, request);
                                    }
                                }
                            }

                            // Only respond to M-SEARCH requests for the correct service type (ST)
                            if request.contains("M-SEARCH") {
                                if let Some(st) = self.extract_st_from_request(&request) {
                                    if st == self.service.st {
                                        println!("Received M-SEARCH request from {}: {}", src, request);
                                        let response = self.create_response();
                                        self.socket.send_to(response.as_bytes(), src).await?;
                                        println!("Sent SSDP response to {}", src);
                                    } else {
                                       
                                    }
                                }
                            }
                        }
                        Err(e) => eprintln!("Error receiving SSDP request: {}", e),
                    }
                }
                _ = time::sleep(self.get_backoff_time()) => { //Used dynamic backoff time
                    let announcement = self.create_announcement();
                    self.socket.send_to(announcement.as_bytes(), multicast_addr).await?;
                    println!("Sent SSDP announcement");

                    // Increment the announcement counter
                    self.announcement_counter += 1;
                }
            }
        }

        Ok(())
    }
    fn extract_nt_from_request(&self, request: &str) -> Option<String> {
        for line in request.lines() {
            if line.starts_with("NT: ") {
                return Some(line[4..].to_string());
            }
        }
        None
    }
    // Extract ST (Search Target) from the request
    fn extract_st_from_request(&self, request: &str) -> Option<String> {
        for line in request.lines() {
            if line.starts_with("ST: ") {
                return Some(line[4..].to_string());
            }
        }
        None
    }

    fn get_backoff_time(&self) -> Duration {
        let base_backoff = 30.0;  // Starting from 30 seconds
        let max_backoff = 120.0;  // Max 2 minutes
        let backoff_time = base_backoff * (self.announcement_counter as f64).ln().max(1.0);
        let backoff_time = backoff_time.min(max_backoff).max(base_backoff);

        Duration::from_secs(backoff_time as u64)
    }


    fn create_response(&self) -> String {
        format!(
            "HTTP/1.1 200 OK\r\n\
            CACHE-CONTROL: max-age=1800\r\n\
            DATE: {}\r\n\
            EXT: \r\n\
            LOCATION: {}\r\n\
            SERVER: SSDP Test Server\r\n\
            ST: {}\r\n\
            USN: {}\r\n\r\n",
            chrono::Utc::now(),
            self.service.location,
            self.service.st,
            self.service.usn
        )
    }

    fn create_announcement(&self) -> String {
        format!(
            "NOTIFY * HTTP/1.1\r\n\
            HOST: 239.255.255.250:1900\r\n\
            NT: {}\r\n\
            NTS: ssdp:alive\r\n\
            LOCATION: {}\r\n\
            USN: {}\r\n\
            CACHE-CONTROL: max-age=1800\r\n\r\n",
            self.service.st,
            self.service.location,
            self.service.usn
        )
    }

}
fn get_local_ip() -> Option<Ipv4Addr> {
    // Use std::net::UdpSocket (synchronous version)
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:53").ok()?; 
    if let Ok(local_addr) = socket.local_addr() {
        if let std::net::SocketAddr::V4(v4_addr) = local_addr {
            return Some(*v4_addr.ip());
        }
    }
    None 
}