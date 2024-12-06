// hash.rs

use sha2::{Sha256, Sha512, Digest};
use base64::{Engine as _, engine::general_purpose};
pub struct HashPeerID;

impl HashPeerID {
    pub fn generate(input: &str, hash_type: &str) -> String {
        match hash_type {
            "sha256" => {
                let mut hasher = Sha256::new();
                hasher.update(input);
                general_purpose::STANDARD.encode(hasher.finalize())
            }
            "sha512" => {
                let mut hasher = Sha512::new();
                hasher.update(input);
                general_purpose::STANDARD.encode(hasher.finalize())
            }
            _ => panic!("Unsupported hash type"),
        }
    }
}
