use anchor_lang::prelude::*;

// Program State
#[derive(InitSpace)]
#[account]
pub struct GhostConfig {
    pub admin: Pubkey,
    pub backend_pubkey: [u8; 32],
    pub dev_badges_minted: u64,
    pub repo_badges_minted: u32, // Track total repos badges minte by the protocol for devs
    pub vouches_count: u32,      // Track total vouches in the protocol scanned
    pub bump: u8,
}

// Collections state / Dev_Badge
#[derive(InitSpace)]
#[account]
pub struct DevState {
    pub dev_addr: Pubkey,
    pub asset_address: Pubkey,
    pub hashed_username: [u8; 32],
    pub repo_count: u32,
    pub owned_repo_count: u32,
    pub total_stars: u32,
    pub total_commits: u32,
    pub prs_merged: u32,
    pub issues_closed: u32,
    pub followers: u32,
    pub account_age_days: u32,
    pub reputation_level: u8,
    pub verified_repos: u64,
    pub vouch_count: u64,
    pub last_updated: i64,
    pub bump: u8,
    pub collection_asset_bump: u8,
}

// Assets state / Repo_badge
#[derive(InitSpace)]
#[account]
pub struct RepoState {
    pub owner: Pubkey,
    pub dev_badge: Pubkey,
    pub hashed_username: [u8; 32],
    #[max_len(50)]
    pub repo_name: Vec<u8>,
    pub stars: u32,
    pub commits: u32,
    pub forks: u32,
    pub open_issues: u32,
    pub is_fork: u8, // (0 or 1) where 1 = true
    #[max_len(10)]
    pub lang1: Vec<u8>,
    #[max_len(10)]
    pub lang2: Vec<u8>,
    pub last_updated: i64,
    pub bump: u8,
    pub badge_bump: u8,
}

#[derive(InitSpace)]
#[account]
pub struct VouchRecord {
    pub voucher: Pubkey,   // Who vouched for
    pub voucher_level: u8, // Voucher's level
    pub target: Pubkey,    // Dev the voucher is vouching for
    pub timestamp: i64,    // timestamp
    pub bump: u8,
}
