use axum::{Json, extract::State, http::HeaderMap};
use chrono::{DateTime, Utc};
use futures::future::join_all;
use reqwest::Client;

// Use Models
use crate::{
    GithubUser, api_models::*, get_session, signer::sign_dev_badge_metrics, signer_public_key,
};

pub async fn fetch_github_user(access_token: &str) -> GithubUser {
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

    res
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

// For fetching the oss stats for dev_badge
pub async fn fetch_oss_stats(client: &Client, username: &str, access_token: &str) -> (u32, u32) {
    let pr_response: serde_json::Value = client
        .get(format!(
            "https://api.github.com/search/issues?q=author:{}+type:pr+is:merged",
            username
        ))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "GhostCheck")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let pr_merged = pr_response["total_count"].as_u64().unwrap_or(0) as u32;

    let issues_response = client
        .get(format!(
            "https://api.github.com/search/issues?q=author:{}+type:issue+is:closed",
            username
        ))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "GhostCheck")
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    let issues_closed = issues_response["total_count"].as_u64().unwrap_or(0) as u32;

    (pr_merged, issues_closed)
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
) -> (u32, u32, u32, u32) {
    let repos = fetch_user_repos(&access_token).await;
    let repo_count = repos.len() as u32;

    let owned_repo_count = repos.iter().filter(|repo| !repo.fork).count() as u32;

    let mut stars_count = 0;
    for repo in repos.iter() {
        if repo.owner.login == username {
            stars_count += repo.stargazers_count
        }
    }

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

    (repo_count, owned_repo_count, total_commits, stars_count)
}

// Check the reputation_level of the user
pub async fn get_reputation_level(
    repo_count: u32,
    total_commits: u32,
    account_age_days: u32,
) -> u8 {
    let user_level: u8 = if repo_count >= 100 && total_commits >= 1500 && account_age_days >= 730 {
        5 // Legend
    } else if repo_count >= 50 && total_commits >= 500 && account_age_days >= 365 {
        4 // Architect
    } else if repo_count >= 20 && total_commits >= 200 && account_age_days >= 120 {
        3 // Builder
    } else if repo_count >= 5 && total_commits >= 30 {
        2 // Coder
    } else {
        1 // Ghost
    };

    user_level
}

// /api/metrics/dev/
pub async fn dev_metrics(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    let session_id = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .find_map(|c| c.trim().strip_prefix("session_id="))
        .unwrap_or("");

    if session_id.is_empty() {
        return Json(serde_json::json!({"error": "Not authorized"}));
    }

    let fetched_session = get_session(&state.db, session_id)
        .await
        .expect("Invalid Session id, failed to fetch from db");

    let token_access = fetched_session.access_token;
    let username = fetched_session.username;

    println!("Username Received {}", username);

    // fetch user metrics
    let (repo_count, owned_repo_count, total_commits, stars) =
        compute_dev_metrics(&state.client, &token_access, &username).await;

    println!(
        "Dev Metrics\nUsername: {}\nRepos: {}\nTotal Commits: {}",
        username, repo_count, total_commits
    );

    // Fetch user oss stats
    let (pr_merged, issues_closed) = fetch_oss_stats(&state.client, &username, &token_access).await;

    // fetch user stats
    let gh_user = fetch_github_user(&token_access).await;
    let created: DateTime<Utc> = gh_user.created_at.parse().unwrap();
    let account_age_days = (Utc::now() - created).num_days() as u32;

    // Get dev's reputation level
    let user_level = get_reputation_level(repo_count, total_commits, account_age_days).await;

    // Sign and parse to json
    let (signature_bytes, hashed_username, hashed_message) = sign_dev_badge_metrics(
        &username,
        repo_count,
        total_commits,
        owned_repo_count,
        stars,
        pr_merged,
        issues_closed,
        gh_user.followers,
        account_age_days,
        user_level,
    );

    let public_key_bytes = signer_public_key();

    Json(serde_json::json!({
        "hashed_username": hashed_username,
        "repo_count": repo_count,
        "owned_repo_count": owned_repo_count,
        "total_stars": stars,
        "total_commit": total_commits,
        "prs_merged": pr_merged,
        "issues_closed": issues_closed,
        "followers": gh_user.followers,
        "account_age_days": account_age_days,
        "reputation_level": user_level,
        "signature": signature_bytes,
        "public_key_bytes": public_key_bytes,
        "signed_message": hashed_message,
    }))
}
