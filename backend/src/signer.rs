use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::env;

pub fn sign_dev_badge_metrics(
    username: &str,
    repo_count: u32,
    total_commits: u32,
) -> (Vec<u8>, [u8; 32]) {
    let secret_hex = env::var("GhostCheck_Signer_Secret").unwrap();
    let secret_bytes = hex::decode(secret_hex).unwrap();

    let signing_key = SigningKey::from_bytes(&secret_bytes.try_into().expect("Must be 32 bytes"));

    // Pad username to 32 bytes to match Solana program
    let mut username_bytes = [0u8; 32];
    let username_slice = username.as_bytes();
    let len = username_slice.len().min(32);
    username_bytes[..len].copy_from_slice(&username_slice[..len]);

    let mut hash = Sha256::new();
    hash.update(&username_bytes);
    hash.update(repo_count.to_be_bytes());
    hash.update(total_commits.to_be_bytes());
    let message = hash.finalize();

    println!("Hashed Message : {:?}", message);

    (
        signing_key.sign(&message).to_bytes().to_vec(),
        username_bytes,
    )
}

pub fn sign_repo_badge_metrics(
    username: &str,
    repo_name: &str,
    lang1: &Vec<u8>,
    lang2: &Vec<u8>,
    stars: u32,
    commits: u32,
) -> (Vec<u8>, [u8; 32]) {
    let signer_hex = env::var("GhostCheck_Signer_Secret").expect("Error parsing env variable");
    let signer_bytes = hex::decode(signer_hex).unwrap();

    let signing_key = SigningKey::from_bytes(
        &signer_bytes
            .try_into()
            .expect("signer key : Must be 32 bytes"),
    );

    // Make the username 32 bytes to match solana program
    let mut username_bytes = [0u8; 32];
    let username_slice = username.as_bytes();
    let len = username_slice.len().min(32);
    username_bytes[..len].copy_from_slice(&username_slice[..len]);

    // Hash the messages
    let mut message = Vec::new();
    message.extend_from_slice(&username_bytes);
    message.extend_from_slice(repo_name.as_bytes());
    message.extend_from_slice(&lang1);
    message.extend_from_slice(&lang2);
    message.extend_from_slice(&stars.to_be_bytes());
    message.extend_from_slice(&commits.to_be_bytes());

    let hashed_message = Sha256::digest(&message);

    let signature = signing_key.sign(&hashed_message);

    (signature.to_bytes().to_vec(), username_bytes)
}

pub fn signer_public_key() -> Vec<u8> {
    let secret_hex = env::var("GhostCheck_Signer_Secret").unwrap();
    let secret_bytes = hex::decode(secret_hex).unwrap();

    let signing_key = SigningKey::from_bytes(&secret_bytes.try_into().expect("Must be 32 bytes"));
    println!("{:?}", signing_key.verifying_key().to_bytes());
    signing_key.verifying_key().as_bytes().to_vec()
}
