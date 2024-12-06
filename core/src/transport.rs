//transport.rs
// Transport Layer
//? Responsible for Transporting Data between Machines
use std::io;
use std::net::SocketAddr;

mod tcp_transport;
mod udp_transport;


use tcp_transport::TcpTransport;
use udp_transport::UdpTransport;
use crate::record::{PeerManagement,PeerRecord};
#[derive(Clone)]
pub struct NautilusTransport {
    pub tcp: TcpTransport,
    pub udp: UdpTransport,
    peer_manager : PeerManagement,
}

impl NautilusTransport {
    /// Create a new UnifiedTransport instance for the given port.
    pub async fn new(port: u16) -> io::Result<Self> {
        let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();

        // Initialize TCP and UDP transports
        let tcp_transport = TcpTransport::new(addr);
        let udp_transport = UdpTransport::new(addr).await?;


        // Initialize Peer Management
        let peer_manager = PeerManagement::new("KPR.json".to_string());
        peer_manager.load_from_file().await?; // Load peers from cache

        Ok(NautilusTransport {
            tcp: tcp_transport,
            udp: udp_transport,
            peer_manager
        })
    }
    /// Start both TCP and UDP listeners.
    pub async fn start_listeners(&self, mut shutdown_rx: tokio::sync::watch::Receiver<bool>) -> io::Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    
        let tcp = self.tcp.clone();
        let tcp_sender = tx.clone();
        let mut shutdown_rx_tcp = shutdown_rx.clone();
        tokio::spawn(async move {
            if let Err(e) = tcp.listen(tcp_sender, shutdown_rx_tcp).await {
                eprintln!("Error in TCP listener: {}", e);
            }
        });
    
        let udp = self.udp.clone();
        let mut shutdown_rx_udp = shutdown_rx.clone();
        tokio::spawn(async move {
            if let Err(e) = udp.listen(tx).await {
                eprintln!("Error in UDP listener: {}", e);
            }
            while shutdown_rx_udp.changed().await.is_ok() {
                if *shutdown_rx_udp.borrow() {
                    println!("Shutting down UDP listener.");
                    break;
                }
            }
        });
    
        // Handle incoming messages and update peers
        while !*shutdown_rx.borrow() {
            if let Ok(Some((addr, data))) = tokio::select! {
                recv = rx.recv() => Ok(recv),
                _ = shutdown_rx.changed() => Err(()),
            } {
                println!("start_listeners received message from {}: {}", addr, String::from_utf8_lossy(&data));
                let peer_record = PeerRecord {
                    addr,
                    peer_id: None,        // Generate or resolve this if needed
                    public_key: None,     // Set if available
                    is_active: true,
                    last_seen: Some(std::time::Instant::now()),
                };
    
                println!("Attempting to add or update peer: {:?}", peer_record);
    
                self.peer_manager.add_or_update_peer(peer_record).await;
                println!("Peer record updated for {}", addr);
            }
        }
    
        println!("Shutting down listeners.");
        Ok(())
    }

    /// Send a message to a specific peer using TCP or UDP.
    pub async fn send(&self, peer_addr: SocketAddr, data: &[u8]) -> io::Result<()> {
        // Attempt TCP first
        if let Err(e) = self.tcp.send(peer_addr, data).await {
            eprintln!("Failed to send via TCP to {}: {}", peer_addr, e);
        }

        // Attempt UDP
        if let Err(e) = self.udp.send(peer_addr, data).await {
            eprintln!("Failed to send via UDP to {}: {}", peer_addr, e);
        }

        Ok(())
    }

    /// Broadcast a message to all known peers via TCP and UDP.
    pub async fn broadcast(&self, data: &[u8]) -> io::Result<()> {
        // Broadcast via TCP
        if let Err(e) = self.tcp.broadcast(data).await {
            eprintln!("Error broadcasting via TCP: {}", e);
        }

        // Broadcast via UDP
        if let Err(e) = self.udp.broadcast(data).await {
            eprintln!("Error broadcasting via UDP: {}", e);
        }

        Ok(())
    }
pub async fn connect(&self, peer_addr: SocketAddr) -> io::Result<()> {
    self.tcp.connect(peer_addr).await?;

    println!("Successfully connected to peer: {}", peer_addr);

    // Add or update the peer in PeerManagement
    let peer_record = PeerRecord {
        addr: peer_addr,
        peer_id: None,        // Generate or resolve this if needed
        public_key: None,     // Set if available
        is_active: true,
        last_seen: Some(std::time::Instant::now()),
    };

    println!("Adding or updating peer record after connection: {:?}", peer_record);

    // Ensure peer is added to PeerManagement
    self.peer_manager.add_or_update_peer(peer_record).await;
    println!("Peer record updated for {}", peer_addr);

    Ok(())
}

    pub async fn remove_peer(&self, peer_addr: SocketAddr) {
        self.peer_manager
            .remove_peer(&peer_addr.to_string())
            .await;
    }

    /// Save peers to cache during shutdown
    pub async fn save_peers(&self) -> io::Result<()> {
        println!("Dumping peer map before saving:");
        self.peer_manager.debug_dump().await;
    
        if let Err(e) = self.peer_manager.save_to_file().await {
            eprintln!("Error saving peers to file: {}", e);
            return Err(e);
        }
    
        println!("Peers successfully saved.");
        Ok(())
    }

    pub async fn get_peers(&self) -> Vec<String> {
        self.peer_manager.get_all_peers().await
    }

}
