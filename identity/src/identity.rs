use tokio::net::TcpStream;

use crate::keypair::{KeyPair, Algorithm};
use crate::peer_id::{PeerID, PeerIDGeneration};
use crate::cEP::CEP;
use crate::cep_error::CEPError;

#[derive(Clone)]
pub struct Identity {
    peer_id: String,
    key_pair: KeyPair,
}

impl Identity {
    /// Creates a new Identity with specified KeyPair algorithm and PeerID generation method
    pub fn new(
        algo: Option<Algorithm>,
        peer_id_generation: Option<PeerIDGeneration>,
    ) -> Self {
        // Unwrap the algorithm or use RSA as the default
        let algorithm = algo.unwrap_or(Algorithm::RSA);

        // Generate KeyPair
        let key_pair = KeyPair::generate(algorithm);

        // Generate PeerID based on the public key
        let peer_id = PeerID::generate(
            Some(peer_id_generation.unwrap_or(PeerIDGeneration::SHA256)), // Wrap in `Some`
            &key_pair.public_key,
        );

        Self { peer_id, key_pair }
    }

    /// Get the PeerID
    pub fn get_peer_id(&self) -> &str {
        &self.peer_id
    }

    /// Get the KeyPair
    pub fn get_key_pair(&self) -> &KeyPair {
        &self.key_pair
    }

    /// Perform CEP (Contact Exchange Protocol) with a remote peer
    pub async fn perform_cep(
        &self,
        stream: &mut TcpStream,
        expected_peer_id: &str,
    ) -> Result<(), CEPError> {
        CEP::perform(stream, &self.key_pair, &self.peer_id).await
    }
}
