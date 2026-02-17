use crate::{AppState, fetch_github_user, get_session, insert_session};
use axum::{
    Json,
    extract::{Query, State},
    http::{self, HeaderMap},
    response::{AppendHeaders, Redirect},
};
use reqwest::header::SET_COOKIE;
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

    let redirect_uri = urlencoding::encode("http://localhost:3000/api/auth/github/callback");
    let github_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user%20repo",
        client_id, redirect_uri
    );

    println!("\nRedirecting to github url: {}", github_url);
    Redirect::temporary(&github_url)
}

// /api/auth/github/callback
pub async fn github_callback(
    State(state): State<AppState>,
    Query(params): Query<CodeQuery>,
) -> (
    AppendHeaders<[(http::header::HeaderName, String); 1]>,
    Redirect,
) {
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

    // Set cookie and redirect to frontend
    let cookie = format!(
        "session_id={}; HttpOnly; SameSite=None; Secure; Path=/; Max-Age=86400",
        session_id
    );

    println!(
        "Login Successful ! Your session_id: {}\nGo to /metrics/dev?session_id={}",
        session_id, session_id
    );

    (
        AppendHeaders([(SET_COOKIE, cookie)]),
        Redirect::temporary("http://localhost:8080/dashboard"),
    )
}

// /api/auth/check
pub async fn check_auth(
    State(state): State<AppState>,
    header: HeaderMap,
) -> Json<serde_json::Value> {
    let session_id = header
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .find_map(|v| v.trim().strip_prefix("session_id="))
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
