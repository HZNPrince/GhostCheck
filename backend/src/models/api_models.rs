use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct Repo {
    pub name: String,
    pub owner: Owner,
}

#[derive(Deserialize)]
pub struct Owner {
    pub login: String,
}

#[derive(Deserialize)]
pub struct Contributor {
    pub login: String,
    pub contributions: u32,
}

// Used for repo_badge , for owner, and stars
#[derive(Deserialize)]
pub struct RepoInfo {
    pub stargazers_count: u32,
    pub owner: Owner,
}

// To receive payload from GET /metrics/repo?session_id=...&repo=...
#[derive(Deserialize)]
pub struct RepoQuery {
    pub session_id: String,
    pub repo: String,
}

// used for Axum state for sharing database and github client
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub client: reqwest::Client,
}
