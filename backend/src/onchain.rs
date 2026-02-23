// This is the file for fetching all the solana account states for dev_badge and repo_Badge

use std::str::FromStr;

use serde::Serialize;
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

const PROGRAM_ID: &str = "GQsPhnZApw9MY7khsbRLtL5mAGpmMn8wp8CFNDPTxGQr";
const ANCHOR_DISCRIMINATOR_LEN: usize = 8;

#[derive(Serialize)]
pub struct DevBadgeData {
    pub exists: bool,
    pub repo_counts: u32,
    pub owned_repo_counts: u32,
    pub total_stars: u32,
    pub total_commits: u32,
    pub prs_merged: u32,
    pub issues_closed: u32,
    pub followers: u32,
    pub account_age_days: u32,
    pub reputation_level: u8,
    pub verified_repos: u64,
    pub vouch_count: u64,
}

pub async fn fetch_dev_badge(rpc: &RpcClient, wallet_str: &str) -> DevBadgeData {
    let wallet = match Pubkey::from_str(wallet_str) {
        Ok(wallet) => wallet,
        Err(_) => {
            return DevBadgeData {
                exists: false,
                ..default_dev()
            };
        }
    };

    let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();

    let (pda, _bump) = Pubkey::find_program_address(&[b"dev_state", wallet.as_ref()], &program_id);
    let account = match rpc.get_account(&pda).await {
        Ok(account) => account,
        Err(_) => {
            return DevBadgeData {
                exists: false,
                ..default_dev()
            };
        }
    };

    let data = &account.data;

    // offset ( dev_addr (32) + asset_address (32) + hashed_username (32) = 96)
    let o = ANCHOR_DISCRIMINATOR_LEN + 96;

    DevBadgeData {
        exists: true,
        repo_counts: read_u32(data, o),
        owned_repo_counts: read_u32(data, o + 4),
        total_stars: read_u32(data, o + 8),
        total_commits: read_u32(data, o + 12),
        prs_merged: read_u32(data, o + 16),
        issues_closed: read_u32(data, o + 20),
        followers: read_u32(data, o + 24),
        account_age_days: read_u32(data, o + 28),
        reputation_level: data[o + 32],
        verified_repos: read_u64(data, o + 33),
        vouch_count: read_u64(data, o + 41),
    }
}

pub fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
}

pub fn read_u64(data: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap())
}

pub fn default_dev() -> DevBadgeData {
    DevBadgeData {
        exists: false,
        repo_counts: 0,
        owned_repo_counts: 0,
        total_stars: 0,
        total_commits: 0,
        prs_merged: 0,
        issues_closed: 0,
        followers: 0,
        account_age_days: 0,
        reputation_level: 0,
        verified_repos: 0,
        vouch_count: 0,
    }
}

#[derive(Serialize)]
pub struct RepoBadgeData {
    pub exists: bool,
    pub repo_name: String,
    pub stars: u32,
    pub commits: u32,
    pub forks: u32,
    pub open_issues: u32,
    pub is_fork: bool,
    pub lang1: String,
    pub lang2: String,
}

pub async fn fetch_repo_badge(
    rpc: &RpcClient,
    hashed_username: &[u8; 32],
    repo_name: &str,
) -> RepoBadgeData {
    let program_id = Pubkey::from_str(PROGRAM_ID).unwrap();

    let (pda, _bump) = Pubkey::find_program_address(
        &[b"repo_state", hashed_username, repo_name.as_bytes()],
        &program_id,
    );

    let account = match rpc.get_account(&pda).await {
        Ok(account) => account,
        Err(_) => return default_repo(repo_name),
    };

    let data = &account.data;

    // Start cursor after discriminator (8) + owner (32) + dev_badge (32) + hashed_username (32)
    let mut cursor = ANCHOR_DISCRIMINATOR_LEN + 96;

    // Repo_name
    // read the length of the vector first to know how much bytes to skip
    let name_len = read_u32(data, cursor) as usize;
    cursor += 4 + name_len;

    // read metrics
    let stars = read_u32(data, cursor);
    cursor += 4;
    let commits = read_u32(data, cursor);
    cursor += 4;
    let forks = read_u32(data, cursor);
    cursor += 4;
    let open_issues = read_u32(data, cursor);
    cursor += 4;
    let is_fork = data[cursor] == 1;
    cursor += 1;

    // read languages
    let lang1_len = read_u32(data, cursor) as usize;
    cursor += 4;
    let lang_bytes = &data[cursor..cursor + lang1_len];
    let lang1 = String::from_utf8_lossy(lang_bytes).to_string();
    cursor += lang1_len;

    let lang2_len = read_u32(data, cursor) as usize;
    cursor += 4;
    let lang_bytes = &data[cursor..cursor + lang2_len];
    let lang2 = String::from_utf8_lossy(lang_bytes).to_string();

    RepoBadgeData {
        exists: true,
        repo_name: repo_name.to_string(),
        stars,
        commits,
        forks,
        open_issues,
        is_fork,
        lang1,
        lang2,
    }
}

pub fn default_repo(repo_name: &str) -> RepoBadgeData {
    RepoBadgeData {
        exists: false,
        repo_name: repo_name.to_string(),
        stars: 0,
        commits: 0,
        forks: 0,
        open_issues: 0,
        is_fork: false,
        lang1: String::new(),
        lang2: String::new(),
    }
}
