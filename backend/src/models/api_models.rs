use serde::Deserialize;

#[derive(Deserialize)]
pub struct Repo {
    pub name: String,
    pub owner: Owner,
}

#[derive(Deserialize)]
pub struct Owner {
    pub login: String,
}

#[derive(Deserialize)]
pub struct Contributor {
    pub login: String,
    pub contributions: u32,
}
