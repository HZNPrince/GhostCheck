use crate::gh_auth::{github_callback, github_login, root};
use axum::{Router, routing::get};

pub fn create_router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/auth/github", get(github_login))
        .route("/auth/github/callback", get(github_callback))
}
