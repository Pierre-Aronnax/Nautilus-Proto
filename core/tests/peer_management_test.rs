#[cfg(test)]
mod tests {
    use Nautilus_Core::record::{PeerManagement, PeerRecord};
    use std::fs;

    #[tokio::test]
    async fn test_add_or_update_peer() {
        let peer_manager = PeerManagement::new("test_peers.json".to_string());
        let peer = PeerRecord {
            addr: "127.0.0.1:8000".parse().unwrap(),
            peer_id: Some("peer1".to_string()),
            public_key: None,
            is_active: true,
            last_seen: None,
        };

        peer_manager.add_or_update_peer(peer.clone()).await;
        let retrieved_peer = peer_manager.get_peer("peer1").await;
        assert_eq!(retrieved_peer.unwrap().addr, peer.addr);
    }

    #[tokio::test]
    async fn test_remove_peer() {
        let peer_manager = PeerManagement::new("test_peers.json".to_string());
        let peer = PeerRecord {
            addr: "127.0.0.1:8000".parse().unwrap(),
            peer_id: Some("peer1".to_string()),
            public_key: None,
            is_active: true,
            last_seen: None,
        };

        peer_manager.add_or_update_peer(peer.clone()).await;
        peer_manager.remove_peer("peer1").await;
        let retrieved_peer = peer_manager.get_peer("peer1").await;
        assert!(retrieved_peer.is_none());
    }

    #[tokio::test]
    async fn test_load_and_save_peers() {
      let test_file = "test_peers.json";
      let peer_manager = PeerManagement::new(test_file.to_string());
      let peer = PeerRecord {
          addr: "127.0.0.1:8000".parse().unwrap(),
          peer_id: Some("peer1".to_string()),
          public_key: None,
          is_active: true,
          last_seen: None,
      };
  
      peer_manager.add_or_update_peer(peer.clone()).await;
      peer_manager.save_to_file().await.unwrap();
  
      let loaded_peer_manager = PeerManagement::new(test_file.to_string());
      loaded_peer_manager.load_from_file().await.unwrap();
      let retrieved_peer = loaded_peer_manager.get_peer("peer1").await;
      assert_eq!(retrieved_peer.unwrap().addr, peer.addr);
  
      // Cleanup test file
      fs::remove_file(test_file).unwrap();
  }
}
