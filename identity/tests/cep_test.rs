#[cfg(test)]
mod tests {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::io::duplex;
    use identity::{KeyPair, Algorithm, CEP, CEPError};
    use identity::c_ep::{CepMessage, CepResponse};
    use identity::c_ep::cep_message::CepType;
    use prost::Message;
    use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey};
    use base64::{engine::general_purpose, Engine as _};
    use sha2::{Digest, Sha256}; // Added Digest trait import

    #[tokio::test]
    async fn test_identify_phase() -> Result<(), CEPError> {
        let (mut client, mut server) = duplex(1024);

        let keypair = KeyPair::generate(Algorithm::RSA);
        let peer_id = "peer1";
        let nonce = "test_nonce";

        tokio::spawn(async move {
            CEP::identify(&mut client, &keypair, peer_id, nonce).await.unwrap();
        });

        let mut buffer = vec![0; 1024];
        let size = server.read(&mut buffer).await?;
        let received_message = CepMessage::decode(&buffer[..size])?;

        if let Some(cep_type) = received_message.cep_type {
            if let CepType::Identification(identification) = cep_type {
                assert_eq!(identification.peer_id, peer_id);
                assert_eq!(identification.nonce, nonce);
            } else {
                panic!("Unexpected CEP message type");
            }
        } else {
            panic!("CEP message type is None");
        }

        Ok(())
    }
    #[tokio::test]
    async fn test_response_phase() -> Result<(), CEPError> {
        let (mut client, mut server) = duplex(1024);
    
        let keypair = KeyPair::generate(Algorithm::RSA);
        let nonce = "test_nonce";
    
        let keypair_clone = keypair.clone();
    
        tokio::spawn(async move {
            let mut hasher = Sha256::new();
            hasher.update(nonce.as_bytes());
            let hashed_nonce = hasher.finalize();
            let hashed_nonce_base64 = base64::engine::general_purpose::STANDARD.encode(hashed_nonce);
    
            let response = CepResponse {
                peer_id: keypair_clone.public_key.clone(),
                public_key: keypair_clone.public_key.clone(),
                signed_nonce: hashed_nonce_base64,
            };
    
            let message = CepMessage {
                cep_type: Some(CepType::Response(response)),
            };
    
            // Debug the serialized message size
            let mut buffer = Vec::new();
            message.encode(&mut buffer).expect("Encoding failed");
            println!("Encoded message size: {}", buffer.len());
    
            client.write_all(&buffer).await.expect("Failed to write to stream");
        });
    
        let mut buffer = vec![0; 1024];
        let size = server.read(&mut buffer).await.expect("Failed to read from stream");
        println!("Received buffer size: {}", size);
    
        let received_message = CepMessage::decode(&buffer[..size])
            .expect("Failed to decode Protobuf message");
    
        if let Some(cep_type) = received_message.cep_type {
            if let CepType::Response(response) = cep_type {
                assert_eq!(response.peer_id, keypair.public_key);
                let decoded_nonce = base64::engine::general_purpose::STANDARD.decode(response.signed_nonce)
                    .expect("Failed to decode Base64 nonce");
    
                let mut hasher = Sha256::new();
                hasher.update(nonce.as_bytes());
                let expected_hashed_nonce = hasher.finalize();
    
                assert_eq!(decoded_nonce, expected_hashed_nonce.as_slice());
            } else {
                panic!("Unexpected CEP message type");
            }
        } else {
            panic!("CEP message type is None");
        }
    
        Ok(())
    }
    
    #[tokio::test]
    async fn test_verify_response_phase() -> Result<(), CEPError> {
        let (mut client, mut server) = duplex(1024);
    
        let keypair = KeyPair::generate(Algorithm::RSA);
        let nonce = "test_nonce";
    
        // Simulate a valid CEPResponse
        tokio::spawn(async move {
            let signed_nonce = keypair.sign(nonce.as_bytes()).unwrap();
            let response = CepResponse {
                peer_id: keypair.public_key.clone(),
                public_key: keypair.public_key.clone(),
                signed_nonce: base64::encode(signed_nonce),
            };
    
            let message = CepMessage {
                cep_type: Some(CepType::Response(response)),
            };
    
            let mut buffer = Vec::new();
            message.encode(&mut buffer).unwrap();
            println!("Simulated response buffer: {:?}", buffer); // Debugging output
            client.write_all(&buffer).await.unwrap();
        });
    
        // Verify the CEP response
        let mut buffer = vec![0; 1024];
        let n = server.read(&mut buffer).await?;
        println!("Received buffer: {:?}", &buffer[..n]); // Debugging output
    
        let is_valid = CEP::verify_response(&mut server, nonce).await?;
        assert!(is_valid, "Failed to verify the CEP response");
    
        Ok(())
    }
}
