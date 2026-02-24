use gloo_net::http::Request;
use serde::Deserialize;
use web_sys::RequestCredentials;

const BACKEND: &str = "";

#[derive(Deserialize, Debug, Clone)]
pub struct DevMetrics {
    pub hashed_username: Vec<u8>,
    pub repo_count: u32,
    pub owned_repo_count: u32,
    pub total_stars: u32,
    pub total_commit: u32,
    pub prs_merged: u32,
    pub issues_closed: u32,
    pub followers: u32,
    pub account_age_days: u32,
    pub reputation_level: u8,
    pub signature: Vec<u8>,
    pub public_key_bytes: Vec<u8>,
    pub signed_message: Vec<u8>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RepoMetrics {
    pub hashed_username: Vec<u8>,
    pub repo_name_bytes: Vec<u8>,
    pub lang1_bytes: Vec<u8>,
    pub lang2_bytes: Vec<u8>,
    pub stars: u32,
    pub commits: u32,
    pub fork_count: u32,
    pub issues_open_count: u32,
    pub is_fork: u8,
    pub signature: Vec<u8>,
    pub public_key_bytes: Vec<u8>,
    pub signed_message: Vec<u8>,
}

pub async fn fetch_github_metrics() -> Result<DevMetrics, String> {
    let response = Request::get(&format!("{}/api/metrics/dev", BACKEND))
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("Request for dev_matrics stats failed {}", e))?;

    if !response.ok() {
        return Err(format!("Request Failed : {}", response.status()));
    }
    response
        .json::<DevMetrics>()
        .await
        .map_err(|e| format!("Failed to parse error: {:?}", e))
}

pub async fn fetch_repo_metrics(repo_name: &str) -> Result<RepoMetrics, String> {
    let response = Request::get(&format!("{}/api/metrics/repo?repo={}", BACKEND, repo_name))
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.ok() {
        return Err(format!("Request Failed: {}", response.status()));
    }
    response
        .json::<RepoMetrics>()
        .await
        .map_err(|e| format!("Failed to parse response : {}", e))
}

#[derive(Deserialize, Debug, Clone)]
pub struct AuthStatus {
    pub authenticated: bool,
    pub username: Option<String>,
}

pub async fn fetch_auth_status() -> Result<AuthStatus, String> {
    let response = Request::get(&format!("{}/api/auth/check", BACKEND))
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("Auth Check failed {}", e))?;

    response
        .json::<AuthStatus>()
        .await
        .map_err(|e| format!("Failed to parse: {:?}", e))
}

#[derive(Deserialize, Debug, Clone)]
pub struct OnChainDevBadge {
    pub exists: bool,
    pub repo_counts: u32,
    pub owned_repo_counts: u32,
    pub total_stars: u32,
    pub total_commits: u32,
    pub prs_merged: u32,
    pub issues_closed: u32,
    pub followers: u32,
    pub account_age_days: u32,
    pub reputation_level: u8,
    pub verified_repos: u64,
    pub vouch_count: u64,
}

// Get request to backend to check the dev_badge for the user is minted
pub async fn check_dev_badge(wallet: &str) -> Result<OnChainDevBadge, String> {
    // Send the request
    let response = Request::get(&format!(
        "{}/api/onchain/dev_badge?wallet={}",
        BACKEND, wallet
    ))
    .credentials(RequestCredentials::Include)
    .send()
    .await
    .map_err(|e| format!("Fetch Dev Badge call to backend failed: {}", e))?;

    // Check the response status
    if !response.ok() {
        return Err(format!("Request Failed: {}", response.status()));
    }

    // Parse the response
    response
        .json::<OnChainDevBadge>()
        .await
        .map_err(|e| format!("Failed to parse dev_badge response: {}", e))
}

#[derive(Deserialize, Debug, Clone)]
pub struct OnchainRepoBadgeData {
    pub exists: bool,
    pub repo_name: String,
    pub stars: u32,
    pub commits: u32,
    pub forks: u32,
    pub open_issues: u32,
    pub is_fork: bool,
    pub lang1: String,
    pub lang2: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ReposResponse {
    pub repos: Vec<OnchainRepoBadgeData>,
}

// Get request to backend to fetch the repo_badges for the user minted
pub async fn fetch_minted_repos(wallet: &str) -> Result<Vec<OnchainRepoBadgeData>, String> {
    let response = Request::get(&format!("{}/api/onchain/repos?wallet={}", BACKEND, wallet))
        .credentials(RequestCredentials::Include)
        .send()
        .await
        .map_err(|e| format!("Fetch Repos call to backend failed : {}", e))?;

    if !response.ok() {
        return Err(format!("request response :{}", response.status()));
    }

    let body: ReposResponse = response
        .json()
        .await
        .map_err(|e| format!("Error parsing repos {}", e))?;

    Ok(body.repos)
}

// To save minted_repos to db
pub async fn save_repo_mint(wallet: &str, repo_name: &str) -> Result<(), String> {
    let response = Request::post(&format!("{}/api/badges/repo", BACKEND))
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(serde_json::json!({"wallet_address": wallet, "repo_name": repo_name}).to_string())
        .map_err(|e| format!("Failed to build request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("POST failed: {}", e))?;

    if !response.ok() {
        return Err(format!("Save failed: {}", response.status()));
    }

    Ok(())
}

// ---- Protocol Stats (public) ----

#[derive(Deserialize, Debug, Clone)]
pub struct ProtocolStats {
    pub dev_badges_minted: u64,
    pub repo_badges_minted: u32,
    pub vouches_count: u32,
}

pub async fn fetch_protocol_stats() -> Result<ProtocolStats, String> {
    let response = Request::get(&format!("{}/api/stats", BACKEND))
        .send()
        .await
        .map_err(|e| format!("Stats fetch failed: {}", e))?;

    if !response.ok() {
        return Err(format!("Stats request failed: {}", response.status()));
    }
    response
        .json::<ProtocolStats>()
        .await
        .map_err(|e| format!("Failed to parse stats: {:?}", e))
}

// ---- Public search (no auth required) ----

pub async fn search_dev_badge(wallet: &str) -> Result<OnChainDevBadge, String> {
    let response = Request::get(&format!(
        "{}/api/onchain/dev_badge?wallet={}",
        BACKEND, wallet
    ))
    .send()
    .await
    .map_err(|e| format!("Search failed: {}", e))?;

    if !response.ok() {
        return Err(format!("Search failed: {}", response.status()));
    }
    response
        .json::<OnChainDevBadge>()
        .await
        .map_err(|e| format!("Failed to parse: {:?}", e))
}

pub async fn search_repos(wallet: &str) -> Result<Vec<OnchainRepoBadgeData>, String> {
    let response = Request::get(&format!("{}/api/search/repos?wallet={}", BACKEND, wallet))
        .send()
        .await
        .map_err(|e| format!("Repo search failed: {}", e))?;

    if !response.ok() {
        return Err(format!("Repo search failed: {}", response.status()));
    }
    let body: ReposResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {:?}", e))?;
    Ok(body.repos)
}
