use tokio::io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt};
use crate::keypair::KeyPair;
use crate::cep_error::CEPError;
use crate::c_ep::{CepIdentification, CepResponse, CepMessage};
use crate::c_ep::cep_message::CepType;
use prost::Message;
use base64::{engine::general_purpose, Engine as _};
use rsa::{Pkcs1v15Sign, RsaPublicKey};
use rsa::pkcs1::DecodeRsaPublicKey;
use chrono::Utc;
use tokio::net::TcpStream;
pub struct CEP;

impl CEP {
    pub async fn identify<S>(
        stream: &mut S,
        keypair: &KeyPair,
        peer_id: &str,
        nonce: &str,
    ) -> Result<(), CEPError>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let timestamp = Utc::now().to_rfc3339();
        let identification = CepIdentification {
            peer_id: peer_id.to_string(),
            public_key: keypair.public_key.clone(),
            nonce: nonce.to_string(),
            timestamp,
            version: "1.0".to_string(),
        };

        let message = CepMessage {
            cep_type: Some(CepType::Identification(identification)),
        };

        let mut buffer = Vec::new();
        message.encode(&mut buffer)?;
        stream.write_all(&buffer).await?;
        Ok(())
    }

    pub async fn respond(
        stream: &mut TcpStream,
        keypair: &KeyPair,
        received_nonce: &str,
    ) -> Result<(), CEPError> {
        let signed_nonce = keypair.sign(received_nonce.as_bytes())?;
        let response = CepResponse {
            peer_id: keypair.public_key.clone(),
            public_key: keypair.public_key.clone(),
            signed_nonce: base64::engine::general_purpose::STANDARD.encode(signed_nonce),
        };
    
        let message = CepMessage {
            cep_type: Some(CepType::Response(response)),
        };
    
        let mut buffer = Vec::new();
        message.encode(&mut buffer).expect("Encoding failed");
        println!("Encoded response message size: {}", buffer.len());
    
        stream.write_all(&buffer).await?;
        Ok(())
    }

    pub async fn verify_response<S: AsyncRead + Unpin>(
        stream: &mut S,
        expected_nonce: &str,
    ) -> Result<bool, CEPError> {
        let mut buffer = vec![0; 1024];
        let n = stream.read(&mut buffer).await?;
        let message = CepMessage::decode(&buffer[..n])?; // Ensure proper decoding
    
        if let Some(CepType::Response(response)) = message.cep_type {
            let decoded_nonce = base64::decode(response.signed_nonce)?;
            let verifying_key = RsaPublicKey::from_pkcs1_pem(&response.public_key)?;
            verifying_key.verify(
                rsa::Pkcs1v15Sign::new::<sha2::Sha256>(),
                expected_nonce.as_bytes(),
                &decoded_nonce,
            )?;
            Ok(true)
        } else {
            Err(CEPError::InvalidMessageType)
        }
    }
    pub async fn perform(
        stream: &mut TcpStream,
        keypair: &KeyPair,
        peer_id: &str,
    ) -> Result<(), CEPError> {
        let nonce = "random_nonce"; // Replace with actual nonce generation.
    
        // Call identify
        Self::identify(stream, keypair, peer_id, nonce).await?;
    
        // Call respond
        Self::respond(stream, keypair, nonce).await?;
    
        Ok(())
    }
}
