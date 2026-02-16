use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::env;

pub fn sign_dev_badge_metrics(
    username: &str,
    repo_count: u32,
    total_commits: u32,
) -> (Vec<u8>, [u8; 32], Vec<u8>) {
    let secret_hex = env::var("GhostCheck_Signer_Secret").unwrap();
    let secret_bytes = hex::decode(secret_hex).unwrap();

    let signing_key = SigningKey::from_bytes(&secret_bytes.try_into().expect("Must be 32 bytes"));

    // hash username to 32 bytes to match Solana program
    let mut hasher = Sha256::new();
    hasher.update(username);
    let hashed_username: [u8; 32] = hasher.finalize().into();

    let mut hash = Sha256::new();
    hash.update(&hashed_username);
    hash.update(repo_count.to_be_bytes());
    hash.update(total_commits.to_be_bytes());
    let hashed_message = hash.finalize();

    println!("Hashed Message : {:?}", hashed_message);

    (
        signing_key.sign(&hashed_message).to_bytes().to_vec(),
        hashed_username,
        hashed_message.to_vec(),
    )
}

pub fn sign_repo_badge_metrics(
    username: &str,
    repo_name: &str,
    lang1: &Vec<u8>,
    lang2: &Vec<u8>,
    stars: u32,
    commits: u32,
) -> (Vec<u8>, [u8; 32], Vec<u8>) {
    let signer_hex = env::var("GhostCheck_Signer_Secret").expect("Error parsing env variable");
    let signer_bytes = hex::decode(signer_hex).unwrap();

    let signing_key = SigningKey::from_bytes(
        &signer_bytes
            .try_into()
            .expect("signer key : Must be 32 bytes"),
    );

    // Make the username 32 bytes to match solana program
    let mut hasher = Sha256::new();
    hasher.update(username);
    let hashed_username: [u8; 32] = hasher.finalize().into();

    // Hash the messages
    let mut message = Vec::new();
    message.extend_from_slice(&hashed_username);
    message.extend_from_slice(repo_name.as_bytes());
    message.extend_from_slice(&lang1);
    message.extend_from_slice(&lang2);
    message.extend_from_slice(&stars.to_be_bytes());
    message.extend_from_slice(&commits.to_be_bytes());

    let hashed_message = Sha256::digest(&message);

    println!("Hashed Message : {:?}", hashed_message);

    let signature = signing_key.sign(&hashed_message);

    (
        signature.to_bytes().to_vec(),
        hashed_username,
        hashed_message.to_vec(),
    )
}

pub fn signer_public_key() -> Vec<u8> {
    let secret_hex = env::var("GhostCheck_Signer_Secret").unwrap();
    let secret_bytes = hex::decode(secret_hex).unwrap();

    let signing_key = SigningKey::from_bytes(&secret_bytes.try_into().expect("Must be 32 bytes"));
    println!("{:?}", signing_key.verifying_key().to_bytes());
    signing_key.verifying_key().as_bytes().to_vec()
}
