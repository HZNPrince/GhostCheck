use anchor_lang::prelude::*;
use mpl_core::{
    instructions::UpdatePluginV1CpiBuilder,
    types::{Attribute, Attributes, Plugin},
    ID as CORE_PROGRAM_ID,
};

use crate::{
    errors::GhostErrors,
    state::{DevState, GhostConfig, RepoState},
    verify_signature,
};

#[derive(Accounts)]
#[instruction(repo_name_padded: [u8; 32])]
pub struct UpdateRepoBadge<'info> {
    #[account(mut)]
    pub dev: Signer<'info>,

    #[account(
        mut,
        seeds= [b"ghost_config"],
        bump = ghost_config.bump
    )]
    pub ghost_config: Account<'info, GhostConfig>,

    #[account(
        mut,
        seeds = [b"dev_state", dev.key().as_ref()],
        bump = dev_state.bump,
        constraint = dev_state.dev_addr == dev.key() @GhostErrors::IncorrectDevState,
    )]
    pub dev_state: Account<'info, DevState>,

    /// CHECK: This is being verified by the core program
    #[account(
        mut,
        seeds = [b"dev_badge", dev.key().as_ref()],
        bump = dev_state.collection_asset_bump,
    )]
    pub dev_badge: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"repo_state", dev_badge.key().as_ref(), &repo_name_padded],
        bump = repo_state.bump,
        constraint = repo_state.dev_badge == dev_badge.key() @GhostErrors::DevBadgeMismatch,
    )]
    pub repo_state: Account<'info, RepoState>,

    /// CHECK: This is being verified by the core program
    #[account(
        mut,
        seeds = [b"repo_badge", dev_badge.key().as_ref(), &repo_name_padded],
        bump = repo_state.bump
    )]
    pub repo_badge: UncheckedAccount<'info>,

    /// CHECK: instruction sysvar instruction intro account should be passed
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instruction_sysvar: UncheckedAccount<'info>,

    /// CHECK: verified by mpl core instruction
    #[account(address = CORE_PROGRAM_ID)]
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> UpdateRepoBadge<'info> {
    pub fn update_repo_badge(
        &mut self,
        repo_name_padded: [u8; 32],
        username_hashed: [u8; 32],
        stars: u32,
        commits: u32,
        forks: u32,
        open_issues: u32,
        lang1: Vec<u8>,
        lang2: Vec<u8>,
    ) -> Result<()> {
        // Verify the message( updated repo stats ) passed in signed by backend
        verify_signature(&self.instruction_sysvar, &self.ghost_config.backend_pubkey)?;

        //Signer seeds for ghost config to sign the cpi
        let signers_seeds: &[&[&[u8]]] = &[&[b"ghost_config", &[self.ghost_config.bump]]];

        UpdatePluginV1CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.repo_badge.to_account_info())
            .collection(Some(&self.dev_badge.to_account_info()))
            .authority(Some(&self.ghost_config.to_account_info()))
            .payer(&self.dev.to_account_info())
            .system_program(&self.system_program.to_account_info())
            .plugin(Plugin::Attributes(Attributes {
                attribute_list: vec![
                    Attribute {
                        key: String::from("repo_name"),
                        value: String::from_utf8(repo_name_padded.to_vec()).unwrap(),
                    },
                    Attribute {
                        key: String::from("stars"),
                        value: stars.to_string(),
                    },
                    Attribute {
                        key: String::from("commits"),
                        value: commits.to_string(),
                    },
                    Attribute {
                        key: String::from("forks"),
                        value: forks.to_string(),
                    },
                ],
            }))
            .invoke_signed(signers_seeds)?;

        // Get current time
        let time_now = Clock::get()?.unix_timestamp;

        let repo_state = &mut self.repo_state;
        repo_state.repo_name = repo_name_padded.to_vec();
        repo_state.hashed_username = username_hashed;
        repo_state.stars = stars;
        repo_state.commits = commits;
        repo_state.forks = forks;
        repo_state.open_issues = open_issues;
        repo_state.lang1 = lang1;
        repo_state.lang2 = lang2;
        repo_state.last_updated = time_now;

        Ok(())
    }
}
