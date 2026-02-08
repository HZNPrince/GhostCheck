use futures::future::join_all;
use reqwest::Client;

// Use Models
use crate::api_models::*;

pub async fn fetch_github_user(access_token: &str) -> String {
    let client = Client::new();

    let res = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "GhostCheck")
        .send()
        .await
        .unwrap()
        .text()
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

pub async fn compute_dev_metrics(access_token: &str, username: &str) -> (u32, u32) {
    let repos = fetch_user_repos(&access_token).await;
    let repo_count = repos.len() as u32;

    let client = Client::new();

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
