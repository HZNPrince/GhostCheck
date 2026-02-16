use gloo_net::http::Request;
use serde::Deserialize;
use web_sys::RequestCredentials;

const BACKEND: &str = "http://localhost:3000";

#[derive(Deserialize, Debug, Clone)]
pub struct DevMetrics {
    pub hashed_username: Vec<u8>,
    pub repo_count: u32,
    pub total_commit: u32,
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
