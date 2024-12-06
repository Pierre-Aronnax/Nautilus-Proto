// peer_id.rs

mod uuid;
mod hash;

use uuid::UUIDPeerID;
use hash::HashPeerID;


pub enum PeerIDGeneration {
    UUID,
    SHA256,
    SHA512,
}

pub struct PeerID;

impl PeerID {
  /// Generate a PeerID based on the provided algorithm or default to SHA256
  pub fn generate(algorithm: Option<PeerIDGeneration>, input: &str) -> String {
      match algorithm.unwrap_or(PeerIDGeneration::SHA256) {
          PeerIDGeneration::UUID => UUIDPeerID::generate(input),
          PeerIDGeneration::SHA256 => HashPeerID::generate(input, "sha256"),
          PeerIDGeneration::SHA512 => HashPeerID::generate(input, "sha512"),
      }
  }
}