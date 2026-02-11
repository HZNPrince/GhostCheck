use crate::AppState;
use crate::gh_api::{dev_metrics, repo_metrics};
use crate::gh_auth::{github_callback, github_login, root};
use axum::{Router, routing::get};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/auth/github", get(github_login))
        .route("/auth/github/callback", get(github_callback))
        .route("/metrics/dev", get(dev_metrics))
        .route("/metrics/repo", get(repo_metrics))
        .with_state(state)
}
