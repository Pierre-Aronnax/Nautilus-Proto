#[allow(dead_code)]
// udp_transport.rs
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
use std::net::SocketAddr;
use std::collections::HashSet;
use std::io;



#[derive(Clone)]
pub struct UdpTransport {
    peers: Arc<Mutex<HashSet<SocketAddr>>>, // Manage known peers
    socket: Arc<UdpSocket>,                 // UDP socket for communication
}

impl UdpTransport {
    /// Creates a new UdpTransport instance.
    pub async fn new(local_addr: SocketAddr) -> io::Result<Self> {
        let socket = UdpSocket::bind(local_addr).await?;
        println!("UDP socket bound to {}", local_addr);

        Ok(UdpTransport {
            peers: Arc::new(Mutex::new(HashSet::new())),
            socket: Arc::new(socket),
        })
    }

    /// Listen for incoming messages.
    pub async fn listen(&self, sender: mpsc::Sender<(SocketAddr, Vec<u8>)>) -> io::Result<()> {
        let socket = self.socket.clone();
    
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        let message = buf[..len].to_vec();
                        println!("UDP listener received message from {}: {}", addr, String::from_utf8_lossy(&message));
    
                        // Forward the message to the shared channel
                        if let Err(e) = sender.send((addr, message)).await {
                            eprintln!("Failed to forward message from {}: {}", addr, e);
                        }
                    }
                    Err(e) => eprintln!("Error receiving UDP message: {}", e),
                }
            }
        });
    
        Ok(())
    }
    /// Send data to a specific peer.
    pub async fn send(&self, peer_addr: SocketAddr, data: &[u8]) -> io::Result<usize> {
        self.socket.send_to(data, peer_addr).await
    }

    /// Broadcast data to all known peers.
    pub async fn broadcast(&self, data: &[u8]) -> io::Result<()> {
        let peers = self.peers.lock().await;
        for &peer in peers.iter() {
            if let Err(e) = self.socket.send_to(data, peer).await {
                eprintln!("Failed to send to {}: {}", peer, e);
            }
        }
        Ok(())
    }

    /// Remove a peer from the known peers list.
    pub async fn remove_peer(&self, peer_addr: SocketAddr) {
        self.peers.lock().await.remove(&peer_addr);
        println!("Removed peer: {}", peer_addr);
    }

    /// Clear all known peers.
    pub async fn clear_peers(&self) {
        self.peers.lock().await.clear();
        println!("Cleared all peers.");
    }
    
}
