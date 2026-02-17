use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::{env, io::Read};

pub fn sign_dev_badge_metrics(
    username: &str,
    repo_count: u32,
    total_commits: u32,
    original_repos: u32,
    total_stars: u32,
    prs_merged: u32,
    issues_closed: u32,
    followers: u32,
    account_age_days: u32,
    reputation_level: u8,
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
    hash.update(original_repos.to_be_bytes());
    hash.update(total_stars.to_be_bytes());
    hash.update(prs_merged.to_be_bytes());
    hash.update(issues_closed.to_be_bytes());
    hash.update(followers.to_be_bytes());
    hash.update(account_age_days.to_be_bytes());
    hash.update(&[reputation_level]);
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
    fork_counts: u32,
    issues_open_count: u32,
    is_fork: u8,
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
    let mut hasher = Sha256::new();
    hasher.update(hashed_username);
    hasher.update(repo_name.as_bytes());
    hasher.update(&lang1);
    hasher.update(&lang2);
    hasher.update(&stars.to_be_bytes());
    hasher.update(&commits.to_be_bytes());
    hasher.update(fork_counts.to_be_bytes());
    hasher.update(issues_open_count.to_be_bytes());
    hasher.update([is_fork]);
    let hashed_message = hasher.finalize();

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
