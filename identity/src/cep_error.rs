use std::fmt;
use rsa::Error as RsaError;

#[derive(Debug)]
pub enum CEPError {
    IO(String),
    Protobuf(String),
    Verification(String),
    Decode(String),
    Rsa(String), // Add the Rsa variant
    InvalidMessageType,
    VerificationFailed,
}

impl fmt::Display for CEPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CEPError::IO(msg) => write!(f, "I/O Error: {}", msg),
            CEPError::Protobuf(msg) => write!(f, "Protobuf Error: {}", msg),
            CEPError::Verification(msg) => write!(f, "Verification Error: {}", msg),
            CEPError::Decode(msg) => write!(f, "Decode Error: {}", msg),
            CEPError::Rsa(msg) => write!(f, "RSA Error: {}", msg), // Handle the Rsa variant
            CEPError::InvalidMessageType => write!(f, "Invalid Message Type"),
            CEPError::VerificationFailed => write!(f, "Verification Failed"),
        }
    }
}

impl From<std::io::Error> for CEPError {
    fn from(e: std::io::Error) -> Self {
        CEPError::IO(e.to_string())
    }
}

impl From<prost::DecodeError> for CEPError {
    fn from(e: prost::DecodeError) -> Self {
        CEPError::Protobuf(e.to_string())
    }
}

impl From<base64::DecodeError> for CEPError {
    fn from(e: base64::DecodeError) -> Self {
        CEPError::Decode(e.to_string())
    }
}

impl From<String> for CEPError {
    fn from(e: String) -> Self {
        CEPError::Verification(e)
    }
}

impl From<rsa::pkcs1::Error> for CEPError {
    fn from(e: rsa::pkcs1::Error) -> Self {
        CEPError::Decode(e.to_string())
    }
}

impl From<prost::EncodeError> for CEPError {
    fn from(e: prost::EncodeError) -> Self {
        CEPError::Protobuf(e.to_string())
    }
}

impl From<RsaError> for CEPError {
    fn from(e: RsaError) -> Self {
        CEPError::Rsa(e.to_string()) // Correctly handle RSA errors
    }
}
