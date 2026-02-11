use std::collections::HashMap;

use axum::{
    Json,
    extract::{Query, State},
};
use futures::future::join_all;
use reqwest::Client;

// Use Models
use crate::{
    GithubUser, api_models::*, get_session, signer::sign_dev_badge_metrics, signer_public_key,
};

pub async fn fetch_github_user(access_token: &str) -> String {
    let client = Client::new();

    let res: GithubUser = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "GhostCheck")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    res.login
}

pub async fn fetch_user_repos(access_token: &str) -> Vec<Repo> {
    let client = Client::new();

    client
        .get("https://api.github.com/user/repos?per_page=100")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "GhostCheck")
        .send()
        .await
        .unwrap()
        .json::<Vec<Repo>>()
        .await
        .unwrap()
}

pub async fn fetch_commits_for_repo(
    client: &Client,
    access_token: &str,
    owner: &str,
    repo: &str,
    username: &str,
) -> u32 {
    println!("Fetching Commits for: {}/{}", username, repo);
    let url = format!(
        "https://api.github.com/repos/{}/{}/contributors",
        owner, repo
    );

    let contributors = client
        .get(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "GhostCheck")
        .send()
        .await
        .unwrap()
        .json::<Vec<Contributor>>()
        .await
        .unwrap();

    for c in contributors {
        if c.login == username {
            println!("{}  {}", c.login, c.contributions);
            return c.contributions;
        }
    }
    0
}

pub async fn compute_dev_metrics(
    client: &Client,
    access_token: &str,
    username: &str,
) -> (u32, u32) {
    let repos = fetch_user_repos(&access_token).await;
    let repo_count = repos.len() as u32;

    let futures = repos.into_iter().map(|repo| {
        let access_token = access_token.to_string();
        let username = username.to_string();
        let client = client.clone();

        async move {
            fetch_commits_for_repo(
                &client,
                &access_token,
                &repo.owner.login,
                &repo.name,
                &username,
            )
            .await
        }
    });

    let result = join_all(futures).await;

    let total_commits = result.iter().sum();

    (repo_count, total_commits)
}

pub async fn dev_metrics(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<serde_json::Value> {
    let session_id = params.get("session_id").unwrap();
    let fetched_session = get_session(&state.db, session_id)
        .await
        .expect("Invalid Session id, failed to fetch from db");

    let token_access = fetched_session.access_token;
    let username = fetched_session.username;

    println!("Username Received {}", username);

    let (repo_count, total_commits) =
        compute_dev_metrics(&state.client, &token_access, &username).await;

    let dev_stats = format!(
        "Dev Metrics\nUsername: {}\nRepos: {}\nTotal Commits: {}",
        username, repo_count, total_commits
    );
    println!("{}", dev_stats);

    // Sign and parse to json
    let (signature_bytes, padded_user) =
        sign_dev_badge_metrics(&username, repo_count, total_commits);

    let public_key_bytes = signer_public_key();

    Json(serde_json::json!({
        "username_padded": padded_user,
        "repo_count": repo_count,
        "total_commit": total_commits,
        "signature": signature_bytes,
        "public_key_bytes": public_key_bytes,
    }))
}
