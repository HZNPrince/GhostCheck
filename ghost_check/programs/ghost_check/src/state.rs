use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct GhostConfig {
    pub admin: Pubkey,
    pub backend_pubkey: [u8; 32],
    pub dev_collections_count: u64,
    pub nft_minted: u32,
    pub bump: u8,
}

#[derive(InitSpace)]
#[account]
pub struct DevState {
    pub address: Pubkey,
    pub hashed_username: [u8; 32],
    pub total_commits: u32,
    pub repo_count: u32,
    pub verified_repo: u64,
    pub bump: u8,
}
