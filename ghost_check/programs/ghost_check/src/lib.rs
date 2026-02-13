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
        total_commits: u32,
    ) -> Result<()> {
        ctx.accounts.verify_signature()?;
        ctx.accounts
            .mint_collection(&username, repo_count, total_commits, &ctx.bumps)
    }

    pub fn mint_repo_badge(
        ctx: Context<RepoBadge>,
        repo_name_padded: [u8; 32],
        username_padded: [u8; 32],
        stars: u32,
        commits: u32,
        lang1: Vec<u8>,
        lang2: Vec<u8>,
    ) -> Result<()> {
        ctx.accounts.verify_signature()?;
        ctx.accounts.mint_repo_badge(
            repo_name_padded,
            username_padded,
            stars,
            commits,
            lang1,
            lang2,
            &ctx.bumps,
        )
    }
}
