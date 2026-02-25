use crate::{AppState, fetch_github_user, get_session, insert_session};
use axum::{
    Json,
    extract::{Query, State},
    http::{self, HeaderMap},
    response::Redirect,
};
use std::env;

// Models
use crate::auth_models::*;

pub async fn root() -> &'static str {
    "Hello from the GhostCheck Backend"
}

// /api/auth/github
pub async fn github_login() -> Redirect {
    println!("Github Logging : Starting ...");

    let client_id = env::var("GITHUB_CLIENT_ID").unwrap();

    let redirect_uri = urlencoding::encode(
        "https://ghostcheck-production.up.railway.app/api/auth/github/callback",
    );
    let github_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user%20public_repo",
        client_id, redirect_uri
    );

    println!("\nRedirecting to github url: {}", github_url);
    Redirect::temporary(&github_url)
}

// /api/auth/github/callback
pub async fn github_callback(
    State(state): State<AppState>,
    Query(params): Query<CodeQuery>,
) -> Redirect {
    println!("Github reached at callback URL with code : {}", params.code);

    let client_id = env::var("GITHUB_CLIENT_ID").unwrap();
    let client_secret = env::var("GITHUB_CLIENT_SECRET").unwrap();

    let client = &state.client;

    println!("Sending All Three (Client_id, client_secret, code) back to github to complete oauth");
    let token_res: TokenResponse = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id),
            ("client_secret", client_secret),
            ("code", params.code),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let token = token_res.access_token;

    let gh_user = fetch_github_user(&token).await;

    // Add to db
    let session_id = insert_session(&state.db, &token, &gh_user.login)
        .await
        .expect("Error inserting and getting the session id");

    let redirect_url = format!(
        "https://ghostcheck-dev.vercel.app/dashboard?session_id={}",
        session_id
    );
    println!(
        "Login Successful ! Your session_id: {}\nRedirecting to {}",
        session_id, redirect_url
    );

    Redirect::temporary(&redirect_url)
}

// /api/auth/check
pub async fn check_auth(
    State(state): State<AppState>,
    header: HeaderMap,
) -> Json<serde_json::Value> {
    let session_id = header
        .get(http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");

    if session_id.is_empty() {
        return Json(serde_json::json!({"authenticated": false, "username": null}));
    }

    match get_session(&state.db, session_id).await {
        Ok(session) => {
            Json(serde_json::json!({"authenticated": true, "username": session.username}))
        }
        Err(_) => Json(serde_json::json!({"authenticated": false, "username": null})),
    }
}
