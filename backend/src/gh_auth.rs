use crate::create_session;
use axum::{extract::Query, response::Redirect};
use std::env;

// Models
use crate::auth_models::*;

pub async fn root() -> &'static str {
    "Hello from the Rust Backend"
}

pub async fn github_login() -> Redirect {
    dotenv::dotenv().ok();
    println!("Github Logging : Starting ...");

    let client_id = env::var("GITHUB_CLIENT_ID").unwrap();

    let redirect_uri = urlencoding::encode("http://localhost:3000/auth/github/callback");
    let github_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user%20repo",
        client_id, redirect_uri
    );

    println!("\nRedirecting to github url: {}", github_url);
    Redirect::temporary(&github_url)
}

pub async fn github_callback(Query(params): Query<CodeQuery>) -> String {
    dotenv::dotenv().ok();
    println!("Github reached at callback URL with code : {}", params.code);

    let client_id = env::var("GITHUB_CLIENT_ID").unwrap();
    let client_secret = env::var("GITHUB_CLIENT_SECRET").unwrap();

    let client = reqwest::Client::new();

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
    let session_id = create_session(token.clone());

    format!(
        "Login Successful ! Your session_id: {}\nGo to /metrics/dev?session_id={}",
        session_id, session_id
    )
}
