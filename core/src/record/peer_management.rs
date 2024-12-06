// Peer_management.rs

use crate::record::peer_record::PeerRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct PeerManagement {
    known_peers: Arc<Mutex<HashMap<String, PeerRecord>>>, // Peer records keyed by Peer ID
    cache_file: String,                                  // Path to cache file
}

impl PeerManagement {
    /// Create a new PeerManagement instance
    pub fn new(cache_file: String) -> Self {
        Self {
            known_peers: Arc::new(Mutex::new(HashMap::new())),
            cache_file,
        }
    }
    /// Add or update a peer in the management list
    pub async fn add_or_update_peer(&self, peer: PeerRecord) {
        let mut peers = self.known_peers.lock().await;
        let key = peer.peer_id.clone().unwrap_or_else(|| peer.addr.to_string());
    
        println!("Adding or updating peer with key: {}", key);
        peers.insert(key, peer);
    
        println!("Current peers in map: {:?}", *peers);
        // No immediate file save here
    }
    

    /// Remove a peer by ID
    pub async fn remove_peer(&self, peer_id: &str) {
        // Step 1: Remove the peer from the in-memory map
        {
            let mut peers = self.known_peers.lock().await;
            peers.remove(peer_id);
        }
    
        // Step 2: Save peers to file asynchronously
        if let Err(e) = self.save_to_file().await {
            eprintln!("Failed to save peers to file: {}", e);
        }
    }

    /// Load peers from the cache file
    pub async fn load_from_file(&self) -> io::Result<()> {
        let mut file = match File::open(&self.cache_file) {
            Ok(file) => file,
            Err(_) => return Ok(()), // If file doesn't exist, return
        };

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let peers: HashMap<String, PeerRecord> = serde_json::from_str(&content)?;
        let mut known_peers = self.known_peers.lock().await;
        *known_peers = peers;
        Ok(())
    }

    /// Save peers to the cache file
    pub async fn save_to_file(&self) -> io::Result<()> {
        let peers = self.known_peers.lock().await;
        let serialized = serde_json::to_string_pretty(&*peers)?;
    
        // Debug log
        println!("Saving peers to file: {}", serialized);
    
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.cache_file)?;
        file.write_all(serialized.as_bytes())?;
        println!("Peers successfully saved to {}", self.cache_file);
    
        Ok(())
    }

    /// Get a peer by ID
    pub async fn get_peer(&self, peer_id: &str) -> Option<PeerRecord> {
        let peers = self.known_peers.lock().await;
        peers.get(peer_id).cloned()
    }

    pub async fn get_all_peers(&self) -> Vec<String> {
        let peers = self.known_peers.lock().await;
        peers.keys().cloned().collect()
    }

    pub async fn debug_dump(&self) {
        let peers = self.known_peers.lock().await;
        println!("Current peers in memory: {:?}", peers);
    }
}


// Custom serialization and deserialization
impl Serialize for PeerManagement {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: serde::Serializer,
  {
      let peers = tokio::runtime::Handle::current().block_on(self.known_peers.lock());
      peers.serialize(serializer)
  }
}

impl<'de> Deserialize<'de> for PeerManagement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let peers = HashMap::<String, PeerRecord>::deserialize(deserializer)?;
        Ok(Self {
            known_peers: Arc::new(Mutex::new(peers)),
            cache_file: "peer_cache.json".to_string(),
        })
    }
}
