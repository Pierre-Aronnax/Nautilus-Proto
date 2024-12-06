use identity::{Algorithm, KeyPair};

#[test]
fn test_rsa_keypair_generate() {
    // Generate RSA KeyPair
    let keypair = KeyPair::generate(Algorithm::RSA);

    // Check that keys are not empty
    assert!(!keypair.private_key.is_empty(), "Private key is empty");
    assert!(!keypair.public_key.is_empty(), "Public key is empty");

    println!("Private Key: {}", keypair.private_key);
    println!("Public Key: {}", keypair.public_key);
}

#[test]
fn test_rsa_sign_and_verify() {
    // Generate RSA KeyPair
    let keypair = KeyPair::generate(Algorithm::RSA);
    let message = b"Hello, RSA Testing!";

    // Signing
    let signature = keypair
        .sign(message)
        .expect("Failed to sign the message");
    println!("Generated Signature: {:?}", signature);

    // Verifying
    let is_valid = keypair
        .verify(message, &signature)
        .expect("Failed to verify the signature");
    assert!(is_valid, "Signature verification failed");
}
