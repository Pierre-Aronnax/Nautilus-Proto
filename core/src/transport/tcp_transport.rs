#[allow(dead_code)]
//tcp_transport.rs
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration};


#[derive(Clone)]
pub struct TcpTransport {
    peers: Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>>, // Manage multiple connections
    addr: SocketAddr,
}

impl TcpTransport {
    /// Creates a new TcpTransport instance.
    pub fn new(addr: SocketAddr) -> Self {
        TcpTransport {
            peers: Arc::new(Mutex::new(HashMap::new())),
            addr,
        }
    }

    /// Start listening for incoming connections.
    pub async fn listen(
        &self,
        sender: mpsc::Sender<(SocketAddr, Vec<u8>)>,
        mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) -> io::Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        println!("TCP listening on {}", self.addr);
    
        loop {
            tokio::select! {
                // Accept new connections
                Ok((stream, addr)) = listener.accept() => {
                    println!("Accepted connection from {}", addr);
    
                    let stream = Arc::new(Mutex::new(stream));
                    self.peers.lock().await.insert(addr, stream.clone());
    
                    let sender_clone = sender.clone();
                    tokio::spawn(async move {
                        let mut buf = vec![0; 1024];
                        let mut stream = stream.lock().await;
                        while let Ok(len) = stream.read(&mut buf).await {
                            if len == 0 {
                                break; // EOF
                            }
                            let message = buf[..len].to_vec();
                            if sender_clone.send((addr, message)).await.is_err() {
                                eprintln!("Failed to send message to handler for {}", addr);
                            }
                        }
                    });
                }
    
                // Check for shutdown signal
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        println!("Shutting down TCP listener.");
                        break;
                    }
                }
            }
        }
    
        Ok(())
    }
    

    
    /// Connect to a remote peer.
    pub async fn connect(&self, peer_addr: SocketAddr) -> io::Result<()> {
        let stream: TcpStream = TcpStream::connect(peer_addr).await?;
        println!("Connection Initiated to {}", peer_addr);
    
        let stream = Arc::new(Mutex::new(stream));
        self.peers.lock().await.insert(peer_addr, stream);
    
        // Perform the handshake
        match self.handshake_with_peer(peer_addr).await {
            Ok(_) => {
                println!("Handshake successful with {}", peer_addr);
                Ok(())
            }
            Err(e) => {
                // If the handshake fails, remove the peer and return an error
                self.peers.lock().await.remove(&peer_addr);
                eprintln!("Handshake failed with {}: {}", peer_addr, e);
                Err(e)
            }
        }
    }

    /// Reconnect to a peer with exponential backoff.
    pub async fn reconnect_peer(
        &self,
        peer_addr: SocketAddr,
        max_attempts: usize,
    ) -> io::Result<()> {
        let mut attempts = 0;
        let mut delay = Duration::from_secs(1); // Initial delay

        loop {
            match self.connect(peer_addr).await {
                Ok(_) => {
                    println!("Reconnected to {}", peer_addr);
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_attempts {
                        eprintln!(
                            "Failed to reconnect to {} after {} attempts: {}",
                            peer_addr, attempts, e
                        );
                        return Err(e);
                    }
                    eprintln!(
                        "Reconnection attempt {}/{} to {} failed: {}. Retrying in {:?}...",
                        attempts, max_attempts, peer_addr, e, delay
                    );
                    sleep(delay).await;
                    delay = delay.saturating_mul(2); // Exponential backoff
                }
            }
        }
    }
    async fn handshake_with_peer(&self, peer_addr: SocketAddr) -> io::Result<()> {
        let peers = self.peers.lock().await;
        if let Some(peer_stream) = peers.get(&peer_addr) {
            let mut stream = peer_stream.lock().await;

            // Send handshake message
            let handshake_message = b"HANDSHAKE_REQUEST";
            stream.write_all(handshake_message).await?;
            println!("Sent handshake request to {}", peer_addr);

            // Wait for response
            let mut response = vec![0; 1024];
            let len = stream.read(&mut response).await?;
            response.truncate(len);

            if response == b"ACK_HANDSHAKE" {
                println!("Handshake successful with {}", peer_addr);
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "Handshake failed"))
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Peer not connected",
            ))
        }
    }

    /// Send data to a specific peer.
    pub async fn send(&self, peer_addr: SocketAddr, data: &[u8]) -> io::Result<usize> {
        let peers = self.peers.lock().await;
        if let Some(peer_stream) = peers.get(&peer_addr) {
            let mut stream = peer_stream.lock().await;
            stream.write(data).await
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Peer not connected",
            ))
        }
    }

    /// Broadcast data to all connected peers.
    pub async fn broadcast(&self, data: &[u8]) -> io::Result<()> {
        let peers = self.peers.lock().await;
        for (addr, peer_stream) in peers.iter() {
            let mut stream = peer_stream.lock().await;
            if let Err(e) = stream.write_all(data).await {
                eprintln!(
                    "Failed to send to {}: {}. Attempting reconnection...",
                    addr, e
                );
                drop(stream); // Release lock to allow reconnect
                self.reconnect_peer(*addr, 5).await?; // Try reconnecting
            }
        }
        Ok(())
    }


    // Handle incoming messages from a peer.
    async fn handle_peer(
        peers: Arc<Mutex<HashMap<SocketAddr, Arc<Mutex<TcpStream>>>>>,
        stream: Arc<Mutex<TcpStream>>,
        addr: SocketAddr,
        sender: mpsc::Sender<(SocketAddr, Vec<u8>)>,
    ) {
        let mut buf = vec![0; 1024]; // Fixed buffer size of 1024 bytes

        let mut stream_guard = stream.lock().await;

        // Wait for handshake message
        match stream_guard.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed by {} (before handshake)", addr);
                return;
            }
            Ok(len) => {
                buf.truncate(len);
                if buf == b"HANDSHAKE_REQUEST" {
                    println!("Received handshake request from {}", addr);
                    // Respond with handshake acknowledgment
                    if let Err(e) = stream_guard.write_all(b"ACK_HANDSHAKE").await {
                        eprintln!("Failed to send handshake acknowledgment to {}: {}", addr, e);
                        return;
                    }
                } else {
                    eprintln!("Invalid handshake message from {}", addr);
                    return;
                }
            }
            Err(e) => {
                eprintln!("Error during handshake with {}: {}", addr, e);
                return;
            }
        }

        // Handshake successful, proceed with communication
        println!("Handshake completed with {}", addr);
        let mut last_activity = tokio::time::Instant::now();
        println!("{:?}", last_activity);
        loop {
            // Check if we've passed the 60-second inactivity timeout
            if last_activity.elapsed() > Duration::from_secs(60) {
                println!(
                    "Connection to {} has been idle for too long. Closing connection.",
                    addr
                );
                break; // Close the connection
            }

            match stream_guard.read(&mut buf).await {
                Ok(0) => {
                    // Connection closed by the client (EOF reached)
                    println!("Connection closed by {} (EOF reached)", addr);
                    break; // Exit the loop
                }
                Ok(len) => {
                    if len > 0 {
                        buf.truncate(len); // Truncate buffer to actual data length
                        println!(
                            "Received {} bytes from {}: {}",
                            len,
                            addr,
                            String::from_utf8_lossy(&buf)
                        );

                        // Send the data to the message handler
                        if sender.send((addr, buf.clone())).await.is_err() {
                            eprintln!("Failed to send message to handler for {}", addr);
                        }
                        last_activity = tokio::time::Instant::now(); // Reset the last activity time
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Non-blocking error, continue retrying
                    continue;
                }
                Err(ref e) if e.kind() == io::ErrorKind::ConnectionReset => {
                    // The connection was reset by the client
                    println!("Connection reset by peer: {}", addr);
                    break; // Exit the loop if the client disconnects
                }
                Err(e) => {
                    eprintln!("Error reading from {}: {}", addr, e);
                    break;
                }
            }
        }

        // Remove the peer from the map if disconnected
        peers.lock().await.remove(&addr);
        println!("Peer {} removed from the peer map.", addr);
    }
    
    /// Closes all connections and clears the peer map.
    pub async fn close_all(&self) -> io::Result<()> {
        let mut peers = self.peers.lock().await;
        for (addr, peer_stream) in peers.drain() {
            let mut stream = peer_stream.lock().await;
            if let Err(e) = stream.shutdown().await {
                eprintln!("Error closing connection to {}: {}", addr, e);
            }
        }
        println!("All connections closed.");
        Ok(())
    }

    pub async fn get_stream(&self, peer_addr: SocketAddr) -> tokio::io::Result<TcpStream> {
        TcpStream::connect(peer_addr).await
    }

    /// Send data to all connected peers.
    pub async fn send_all(&self, data: &[u8]) -> io::Result<()> {
        let peers = self.peers.lock().await;
        for (addr, peer_stream) in peers.iter() {
            let mut stream = peer_stream.lock().await;
            if let Err(e) = stream.write_all(data).await {
                eprintln!(
                    "Failed to send to {}: {}. Attempting reconnection...",
                    addr, e
                );
                drop(stream); // Release lock to allow reconnect
                self.reconnect_peer(*addr, 5).await?; // Try reconnecting
            }
        }
        Ok(())
    }

    
}
