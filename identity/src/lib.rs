// lib.rs
mod keypair; // generate a keypair
mod peer_id; // generate a peer_id
mod identity; // Unified Access to the Keypair and Peer_ID
mod cep_error;
mod cEP;

pub use peer_id::PeerIDGeneration; // Enum Options for PeerID generation
pub use keypair::{Algorithm,KeyPair}; // Enum Options for the PKI Algo
pub use identity::Identity;
pub use cEP::CEP;
pub use cep_error::CEPError;

pub mod c_ep { // protocol Contact Exchange Protocol
  include!(concat!(env!("OUT_DIR"), "/cEP.rs"));
}