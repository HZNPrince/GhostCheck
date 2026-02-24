use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use serde::Deserialize;

use crate::{
    AppState,
    badges::fetch_all_for_wallet,
    get_session,
    onchain::{fetch_dev_badge, fetch_protocol_stats, fetch_repo_badge},
};

#[derive(Deserialize)]
pub struct WalletQuery {
    pub wallet: String,
}

// GET /api/stats — public, no auth
pub async fn get_protocol_stats(State(state): State<AppState>) -> Json<serde_json::Value> {
    let stats = fetch_protocol_stats(&state.rpc).await;
    Json(serde_json::to_value(stats).unwrap())
}

// GET /api/onchain/dev_badge?wallet=<pubkey> — public
pub async fn get_dev_badge(
    State(state): State<AppState>,
    Query(params): Query<WalletQuery>,
) -> Json<serde_json::Value> {
    let wallet = params.wallet;
    let badge = fetch_dev_badge(&state.rpc, &wallet).await;
    Json(serde_json::to_value(badge).unwrap())
}

// GET /api/onchain/repos?wallet=<pubkey> — requires session (for own repos)
pub async fn get_repo_badges(
    State(state): State<AppState>,
    Query(params): Query<WalletQuery>,
    header: HeaderMap,
) -> Json<serde_json::Value> {
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

    let _session = match get_session(&state.db, session_id).await {
        Ok(s) => s,
        Err(_) => return Json(serde_json::json!({"error": "Invalid session"})),
    };

    let repo_names = match fetch_all_for_wallet(&state.db, &params.wallet).await {
        Ok(repos) => repos,
        Err(_) => return Json(serde_json::json!({"repos": []})),
    };
    let mut repos = Vec::new();
    for repo in repo_names {
        let badge = fetch_repo_badge(&state.rpc, &params.wallet, &repo).await;
        repos.push(badge);
    }

    Json(serde_json::json!({"repos": repos}))
}

// GET /api/search/repos?wallet=<pubkey> — public (for open search)
pub async fn search_repo_badges(
    State(state): State<AppState>,
    Query(params): Query<WalletQuery>,
) -> Json<serde_json::Value> {
    let repo_names = match fetch_all_for_wallet(&state.db, &params.wallet).await {
        Ok(repos) => repos,
        Err(_) => return Json(serde_json::json!({"repos": []})),
    };
    let mut repos = Vec::new();
    for repo in repo_names {
        let badge = fetch_repo_badge(&state.rpc, &params.wallet, &repo).await;
        if badge.exists {
            repos.push(badge);
        }
    }

    Json(serde_json::json!({"repos": repos}))
}
