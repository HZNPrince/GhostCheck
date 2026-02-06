use axum::{extract::Query, response::Redirect};
use serde::Deserialize;
use std::env;

pub async fn root() -> &'static str {
    "Hello from the Rust Backend"
}

pub async fn github_login() -> Redirect {
    dotenv::dotenv().ok();

    let client_id = env::var("GITHUB_CLIENT_ID").unwrap();

    let redirect_uri = urlencoding::encode("http://localhost:3000/auth/github/callback");
    let github_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user%20repo",
        client_id, redirect_uri
    );

    Redirect::temporary(&github_url)
}

#[derive(Deserialize)]
pub struct CodeQuery {
    code: String,
}

pub async fn github_callback(Query(params): Query<CodeQuery>) -> String {
    dotenv::dotenv().ok();
    let client_id = env::var("GITHUB_CLIENT_ID").unwrap();
    let client_secret = env::var("GITHUB_CLIENT_SECRET").unwrap();

    let client = reqwest::Client::new();

    let res = client
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
        .text()
        .await
        .unwrap();

    format!("Github responded with: {}", res)
}
