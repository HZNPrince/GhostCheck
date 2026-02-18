use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;

pub mod errors;

pub mod state;

declare_id!("GQsPhnZApw9MY7khsbRLtL5mAGpmMn8wp8CFNDPTxGQr");

#[program]
pub mod ghost_check {
    use super::*;

    pub fn init_config(ctx: Context<InitConfig>, backend_pubkey: [u8; 32]) -> Result<()> {
        ctx.accounts.init_config(backend_pubkey, &ctx.bumps)
    }

    pub fn mint_dev_badge(
        ctx: Context<DevBadge>,
        username: [u8; 32],
        repo_count: u32,
        owned_repo_count: u32,
        total_stars: u32,
        total_commits: u32,
        prs_merged: u32,
        issues_closed: u32,
        followers: u32,
        account_age_days: u32,
        reputation_level: u8,
    ) -> Result<()> {
        ctx.accounts.mint_collection(
            &username,
            repo_count,
            owned_repo_count,
            total_stars,
            total_commits,
            prs_merged,
            issues_closed,
            followers,
            account_age_days,
            reputation_level,
            &ctx.bumps,
        )
    }

    pub fn mint_repo_badge(
        ctx: Context<RepoBadge>,
        repo_name_padded: [u8; 32],
        username_padded: [u8; 32],
        stars: u32,
        commits: u32,
        forks: u32,
        open_issues: u32,
        is_fork: u8,
        lang1: Vec<u8>,
        lang2: Vec<u8>,
    ) -> Result<()> {
        ctx.accounts.mint_repo_badge(
            repo_name_padded,
            username_padded,
            stars,
            commits,
            forks,
            open_issues,
            is_fork,
            lang1,
            lang2,
            &ctx.bumps,
        )
    }

    pub fn update_dev_badge(
        ctx: Context<UpdateDevBadge>,
        username: [u8; 32],
        repo_count: u32,
        owned_repo_count: u32,
        total_stars: u32,
        total_commits: u32,
        prs_merged: u32,
        issues_closed: u32,
        followers: u32,
        account_age_days: u32,
        reputation_level: u8,
    ) -> Result<()> {
        ctx.accounts.update_dev_badge(
            &username,
            repo_count,
            owned_repo_count,
            total_stars,
            total_commits,
            prs_merged,
            issues_closed,
            followers,
            account_age_days,
            reputation_level,
        )
    }

    pub fn update_repo_badge(
        ctx: Context<UpdateRepoBadge>,
        repo_name_padded: [u8; 32],
        username_hashed: [u8; 32],
        stars: u32,
        commits: u32,
        forks: u32,
        open_issues: u32,
        lang1: Vec<u8>,
        lang2: Vec<u8>,
    ) -> Result<()> {
        ctx.accounts.update_repo_badge(
            repo_name_padded,
            username_hashed,
            stars,
            commits,
            forks,
            open_issues,
            lang1,
            lang2,
        )
    }

    pub fn verify_dev(ctx: Context<VerifyDev>, dev_addr: [u8; 32], min_lvl: u8) -> Result<()> {
        ctx.accounts.verify_dev(dev_addr, min_lvl)
    }

    pub fn vouch_for_dev(ctx: Context<Vouch>, target_addr: [u8; 32]) -> Result<()> {
        ctx.accounts.vouch_for_dev(target_addr, &ctx.bumps)
    }
}
