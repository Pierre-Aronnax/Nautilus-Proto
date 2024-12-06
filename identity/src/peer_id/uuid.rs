// uuid.rs

use uuid::Uuid;

pub struct UUIDPeerID;

impl UUIDPeerID {
    pub fn generate(_: &str) -> String {
        Uuid::new_v4().to_string()
    }
}
