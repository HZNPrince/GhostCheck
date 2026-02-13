use anchor_lang::prelude::*;

// Program State
#[derive(InitSpace)]
#[account]
pub struct GhostConfig {
    pub admin: Pubkey,
    pub backend_pubkey: [u8; 32],
    pub dev_collections_count: u64,
    pub nft_minted: u32,
    pub bump: u8,
}

// Collections state / Dev_Badge
#[derive(InitSpace)]
#[account]
pub struct DevState {
    pub address: Pubkey,
    pub hashed_username: [u8; 32],
    pub total_commits: u32,
    pub repo_count: u32,
    pub verified_repo: u64,
    pub bump: u8,
    pub collection_asset_bump: u8,
}

// Assets state / Repo_badge
#[derive(InitSpace)]
#[account]
pub struct RepoState {
    pub owner: Pubkey,
    pub hashed_username: [u8; 32],
    #[max_len(32)]
    pub repo_name: Vec<u8>,
    pub dev_badge: Pubkey,
    pub stars: u32,
    pub commits: u32,
    #[max_len(10)]
    pub lang1: Vec<u8>,
    #[max_len(10)]
    pub lang2: Vec<u8>,
}
