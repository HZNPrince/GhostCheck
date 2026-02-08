use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use std::env;

pub fn sign_dev_badge_metrics(username: &str, repo_count: u32, total_commits: u32) -> String {
    dotenv::dotenv().ok();
    let secret_hex = env::var("GhostCheck_Signer_Secret").unwrap();
    let secret_bytes = hex::decode(secret_hex).unwrap();

    let signing_key = SigningKey::from_bytes(&secret_bytes.try_into().expect("Must be 32 bytes"));

    let mut hash = Sha256::new();
    hash.update(username.as_bytes());
    hash.update(repo_count.to_be_bytes());
    hash.update(total_commits.to_be_bytes());
    let message = hash.finalize();

    let signature = signing_key.sign(&message);

    hex::encode(signature.to_bytes())
}
