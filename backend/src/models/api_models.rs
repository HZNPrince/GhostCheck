use serde::Deserialize;
use sqlx::PgPool;
// used for fetch_user_repos  for dev stats
#[derive(Deserialize)]
pub struct Repo {
    pub name: String,
    pub owner: Owner,
    pub fork: bool,
    pub stargazers_count: u32,
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

// Used for repo_badge , for fetching single repo
#[derive(Deserialize)]
pub struct RepoInfo {
    pub stargazers_count: u32,
    pub owner: Owner,
    pub forks_count: u32,
    pub fork: bool,
    pub open_issues_count: u32,
}

// To receive payload from GET /metrics/repo?session_id=...&repo=...
#[derive(Deserialize)]
pub struct RepoQuery {
    pub repo: String,
}

// used for Axum state for sharing database and github client
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub client: reqwest::Client,
}
