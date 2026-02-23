use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    AppState,
    badges::fetch_all_for_wallet,
    get_session,
    onchain::{fetch_dev_badge, fetch_repo_badge},
};

#[derive(Deserialize)]
pub struct WalletQuery {
    pub wallet: String,
}

// To get if user minted_dev_badge yet
// GET /api/onchain/dev-badge?wallet=<pubkey>
pub async fn get_dev_badge(
    State(state): State<AppState>,
    Query(params): Query<WalletQuery>,
) -> Json<serde_json::Value> {
    let wallet = params.wallet;
    let badge = fetch_dev_badge(&state.rpc, &wallet).await;

    Json(serde_json::to_value(badge).unwrap())
}

// To get all the minted repos of user
// GET /api/onchain/repos?wallet=<pubkey>
pub async fn get_repo_badges(
    State(state): State<AppState>,
    Query(params): Query<WalletQuery>,
    header: HeaderMap,
) -> Json<serde_json::Value> {
    // Get the username from session_id
    let session_id = header
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .find_map(|c| c.trim().strip_prefix("session_id="))
        .unwrap_or("");

    if session_id.is_empty() {
        return Json(serde_json::json!({"error": "Not Authorized"}));
    }

    let session = match get_session(&state.db, session_id).await {
        Ok(s) => s,
        Err(_) => return Json(serde_json::json!({"error": "Invalid session"})),
    };

    let mut hasher = Sha256::new();
    hasher.update(session.username);
    let hashed_username: [u8; 32] = hasher.finalize().into();

    // Get repo_names from db
    let repo_names = match fetch_all_for_wallet(&state.db, &params.wallet).await {
        Ok(repos) => repos,
        Err(_) => return Json(serde_json::json!({"repos": []})),
    };
    let mut repos = Vec::new();
    for repo in repo_names {
        let badge = fetch_repo_badge(&state.rpc, &hashed_username, &repo).await;
        repos.push(badge);
    }

    Json(serde_json::json!({"repos": repos}))
}
