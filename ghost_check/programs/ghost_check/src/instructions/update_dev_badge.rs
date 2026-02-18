use anchor_lang::prelude::*;

use crate::{
    errors::GhostErrors,
    state::{DevState, GhostConfig},
    verify_signature,
};

#[derive(Accounts)]
pub struct UpdateDevBadge<'info> {
    #[account(mut)]
    pub dev: Signer<'info>,

    #[account(
        mut,
        seeds = [b"ghost_config"],
        bump = ghost_config.bump,
    )]
    pub ghost_config: Account<'info, GhostConfig>,

    #[account(
        mut,
        seeds = [b"dev_state", dev.key().as_ref()],
        bump = dev_state.bump,
        constraint = dev.key() == dev_state.dev_addr @GhostErrors::IncorrectDevState
    )]
    pub dev_state: Account<'info, DevState>,

    /// CHECK: This is being verifed by the contraints and by the core program
    #[account(
        mut,
        seeds = [b"dev_badge", dev.key().as_ref()],
        bump = dev_state.collection_asset_bump,
        address = dev_state.asset_address,
    )]
    pub dev_badge: UncheckedAccount<'info>,

    /// CHECK: instruction sysvar instruction intro account should be passed
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instruction_sysvar: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateDevBadge<'info> {
    pub fn update_dev_badge(
        &mut self,
        username: &[u8; 32],
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
        // Verify the message( updated dev stats ) passed in signed by backend
        verify_signature(
            &self.instruction_sysvar.to_account_info(),
            &self.ghost_config.backend_pubkey,
        )?;

        // Get Current Timestamp
        let time_now = Clock::get()?.unix_timestamp;

        let dev_state = &mut self.dev_state;
        dev_state.hashed_username = username.clone();
        dev_state.repo_count = repo_count;
        dev_state.owned_repo_count = owned_repo_count;
        dev_state.total_stars = total_stars;
        dev_state.total_commits = total_commits;
        dev_state.prs_merged = prs_merged;
        dev_state.issues_closed = issues_closed;
        dev_state.followers = followers;
        dev_state.account_age_days = account_age_days;
        dev_state.reputation_level = reputation_level;
        dev_state.last_updated = time_now;

        Ok(())
    }
}
