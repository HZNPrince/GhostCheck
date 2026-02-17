use serde::Deserialize;

#[derive(Deserialize)]
pub struct CodeQuery {
    pub code: String,
}
#[derive(Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

// For auth , for fetch gh user stats
#[derive(Deserialize)]
pub struct GithubUser {
    pub login: String,
    pub followers: u32,
    pub created_at: String,
}
