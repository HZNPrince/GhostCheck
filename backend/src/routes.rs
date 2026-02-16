use crate::AppState;
use crate::gh_api::{dev_metrics, repo_metrics};
use crate::gh_auth::{check_auth, github_callback, github_login, root};
use axum::http;
use axum::{Router, routing::get};
use reqwest::{Method, header};
use tower_http::cors::CorsLayer;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:8080"
                .parse::<http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods(Method::GET)
        .allow_headers([header::CONTENT_TYPE, header::COOKIE])
        .allow_credentials(true);

    Router::new()
        .route("/", get(root))
        .route("/api/auth/github", get(github_login))
        .route("/api/auth/github/callback", get(github_callback))
        .route("/api/auth/check", get(check_auth))
        .route("/api/metrics/dev", get(dev_metrics))
        .route("/api/metrics/repo", get(repo_metrics))
        .layer(cors)
        .with_state(state)
}
