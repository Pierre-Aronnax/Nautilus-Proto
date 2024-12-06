
mod rsa_keypair;

use rsa_keypair::RSAKeyPair;

#[derive(Clone, Debug)]
pub enum Algorithm {
    RSA,
    ECDSA,
    SECP256k1,
    NTRU,
}

#[derive(Clone)]
pub struct KeyPair {
    pub public_key: String,
    pub private_key: String,
    algorithm: Algorithm,
}

impl KeyPair {
    /// Generate a new KeyPair using the specified algorithm
    pub fn generate(algorithm: Algorithm) -> Self {
        match algorithm {
            Algorithm::RSA => RSAKeyPair::generate(),
            Algorithm::ECDSA => unimplemented!("ECDSA generation is not implemented."),
            Algorithm::SECP256k1 => unimplemented!("SECP256k1 generation is not implemented."),
            Algorithm::NTRU => unimplemented!("NTRU generation is not implemented."),
        }
    }

    /// Sign a message using the KeyPair
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, String> {
        match self.algorithm {
            Algorithm::RSA => RSAKeyPair::sign(self, message),
            _ => Err(format!("Signing not implemented for {:?}", self.algorithm)),
        }
    }

    /// Verify a signature using the KeyPair
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool, String> {
        match self.algorithm {
            Algorithm::RSA => RSAKeyPair::verify(self, message, signature),
            _ => Err(format!("Verification not implemented for {:?}", self.algorithm)),
        }
    }
}
