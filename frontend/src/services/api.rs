use gloo_net::http::Request;
use serde::Deserialize;

const BACKEND_URL: &str = "http://localhost:3000";

#[derive(Deserialize, Debug, Clone)]
pub struct DevMetrics {
    pub username_padded: Vec<u8>,
    pub repo_count: u32,
    pub total_commit: u32,
    pub signature: Vec<u8>,
    pub public_key_bytes: Vec<u8>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RepoMetrics {
    pub user_name_padded: Vec<u8>,
    pub repo_name_padded: Vec<u8>,
    pub lang1_bytes: Vec<u8>,
    pub lang2_bytes: Vec<u8>,
    pub stars: u32,
    pub commits: u32,
    pub signature: Vec<u8>,
    pub public_key_bytes: Vec<u8>,
}

pub async fn fetch_github_metrics(session_id: &str) -> Result<DevMetrics, String> {
    let url = format!("{}/metrics/dev?session_id={}", BACKEND_URL, session_id);
    let response = Request::get(&url)
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

pub async fn fetch_repo_metrics(session_id: &str, repo_name: &str) -> Result<RepoMetrics, String> {
    let url = format!(
        "{}/metrics/repo?session_id={}&repo={}",
        BACKEND_URL, session_id, repo_name
    );

    let response = Request::get(&url)
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
