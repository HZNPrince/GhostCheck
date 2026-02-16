use crate::{get_session, models::api_models::*, sign_repo_badge_metrics, signer_public_key};
use anyhow;
use axum::{
    Json,
    extract::{Query, State},
    http::HeaderMap,
};
use reqwest::Client;
use std::collections::HashMap;

pub async fn fetch_repo_metrics(
    client: &Client,
    token_access: &str,
    username: &str,
    repo_name: &str,
) -> anyhow::Result<(u32, Vec<u8>, Vec<u8>, u32)> {
    // returning (stars , lang1 , option<lang2> , commits)

    // fetch the stargazers_count and owner
    let repo_info: RepoInfo = client
        .get(format!(
            "https://api.github.com/repos/{}/{}",
            username, repo_name
        ))
        .header("Authorization", format!("Bearer {}", token_access))
        .header("User-Agent", "GhostCheck")
        .send()
        .await?
        .json()
        .await
        .map_err(|e| {
            anyhow::anyhow!("RepoStats fetch failed : Did you enter the repo name correctly ?")
        })?;

    // Validate owner and store the stars
    if repo_info.owner.login != username {
        anyhow::bail!("User is not owner of repo!");
    }
    let stars = repo_info.stargazers_count;

    // To fetch the language of the repo
    let languages: HashMap<String, u64> = client
        .get(format!(
            "https://api.github.com/repos/{}/{}/languages",
            username, repo_name
        ))
        .header("Authorization", format!("Bearer {}", token_access))
        .header("User-Agent", "GhostCheck")
        .send()
        .await?
        .json()
        .await?;

    // Sort the language by bytes
    let mut langs: Vec<_> = languages.into_iter().collect();
    langs.sort_by(|a, b| b.1.cmp(&a.1));

    let lang1 = langs
        .get(0)
        .map(|l| l.0.as_bytes().to_vec())
        .unwrap_or_default();
    let lang2 = langs
        .get(1)
        .map(|l| l.0.as_bytes().to_vec())
        .unwrap_or_default();

    // Fetch Commits by user
    let contributor: Vec<Contributor> = client
        .get(format!(
            "https://api.github.com/repos/{}/{}/contributors",
            username, repo_name
        ))
        .header("Authorization", format!("Bearer {}", token_access))
        .header("User-Agent", "GhostCheck")
        .send()
        .await?
        .json()
        .await?;

    let commits = contributor
        .into_iter()
        .find(|n| n.login == username)
        .map(|c| c.contributions)
        .unwrap_or(0);

    Ok((stars, lang1, lang2, commits))
}

pub async fn repo_metrics(
    State(state): State<AppState>,
    Query(params): Query<RepoQuery>,
    headers: HeaderMap,
) -> Json<serde_json::Value> {
    let session_id = headers
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .find_map(|s| s.trim().strip_prefix("session_id="))
        .unwrap_or("");

    if session_id.is_empty() {
        return Json(serde_json::json!({
            "error": "Not authorized"
        }));
    }

    let session = get_session(&state.db, session_id)
        .await
        .expect("Error Fetching session from db");

    let access_token = session.access_token;
    let username = session.username;

    let (stars, lang1, lang2, commits) =
        fetch_repo_metrics(&state.client, &access_token, &username, &params.repo)
            .await
            .expect("Error fetching repo stats");

    //Sign the metrics
    let (signature, padded_user, hashed_message) =
        sign_repo_badge_metrics(&username, &params.repo, &lang1, &lang2, stars, commits);

    let public_key = signer_public_key();

    Json(serde_json::json!({
        "hashed_username": padded_user,
        "repo_name_bytes": params.repo.as_bytes(),
        "lang1_bytes": lang1,
        "lang2_bytes": lang2,
        "stars": stars,
        "commits": commits,
        "signature": signature,
        "public_key_bytes": public_key,
        "signed_message": hashed_message,
    }))
}
