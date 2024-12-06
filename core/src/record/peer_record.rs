// peer_record.rs

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerRecord {
    pub addr: SocketAddr,
    pub peer_id: Option<String>,
    pub public_key: Option<String>,
    pub is_active: bool,
    #[serde(
        serialize_with = "serialize_instant",
        deserialize_with = "deserialize_instant"
    )]
    pub last_seen: Option<std::time::Instant>,
}

// Helper functions for serialization
fn serialize_instant<S>(instant: &Option<std::time::Instant>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match instant {
        Some(instant) => serializer.serialize_some(&instant.elapsed().as_secs()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_instant<'de, D>(deserializer: D) -> Result<Option<std::time::Instant>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let secs: Option<u64> = Option::deserialize(deserializer)?;
    Ok(secs.map(|s| std::time::Instant::now() - std::time::Duration::from_secs(s)))
}
