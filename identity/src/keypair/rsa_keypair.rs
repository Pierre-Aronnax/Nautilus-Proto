use rsa::{
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
    pkcs1v15::{SigningKey, VerifyingKey},
    signature::{RandomizedSigner, SignatureEncoding, Verifier},
    RsaPrivateKey, RsaPublicKey,
};
use sha2::Sha256;
use zeroize::Zeroizing;
use crate::keypair::{KeyPair, Algorithm};

pub struct RSAKeyPair;

impl RSAKeyPair {
    /// Generate an RSA KeyPair
    pub fn generate() -> KeyPair {
        let mut rng = rand_core::OsRng;
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate private key");
        let public_key = private_key.to_public_key();

        let private_key_pem = Zeroizing::new(
            private_key
                .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
                .expect("Failed to convert private key to PEM"),
        );

        let public_key_pem = public_key
            .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
            .expect("Failed to convert public key to PEM");

        KeyPair {
            public_key: public_key_pem,
            private_key: private_key_pem.to_string(),
            algorithm: Algorithm::RSA,
        }
    }

    /// Sign a message using RSA
    pub fn sign(keypair: &KeyPair, message: &[u8]) -> Result<Vec<u8>, String> {
        let private_key = RsaPrivateKey::from_pkcs1_pem(&keypair.private_key)
            .map_err(|e| format!("Failed to parse private key: {}", e))?;
        let signing_key = SigningKey::<Sha256>::new(private_key);

        let mut rng = rand_core::OsRng;
        let signature = signing_key
            .sign_with_rng(&mut rng, message)
            .to_bytes()
            .to_vec();

        Ok(signature)
    }

    /// Verify a signature using RSA
    pub fn verify(keypair: &KeyPair, message: &[u8], signature: &[u8]) -> Result<bool, String> {
        let public_key = RsaPublicKey::from_pkcs1_pem(&keypair.public_key)
            .map_err(|e| format!("Failed to parse public key: {}", e))?;
        let verifying_key = VerifyingKey::<Sha256>::new(public_key);

        let rsa_signature = rsa::pkcs1v15::Signature::try_from(signature)
            .map_err(|e| format!("Invalid signature format: {}", e))?;

        verifying_key
            .verify(message, &rsa_signature)
            .map(|_| true)
            .map_err(|e| format!("Verification failed: {}", e))
    }
}
