use crate::errors::GhostErrors;
use crate::state::{DevState, GhostConfig, RepoState};
use crate::verify_signature;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::ID as SYSVAR_INSTRUCTION_ID;
use mpl_core::types::{
    Attribute, Attributes, PermanentFreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair,
    UpdateDelegate,
};
use mpl_core::{instructions::CreateV2CpiBuilder, ID as CORE_PROGRAM_ID};

#[derive(Accounts)]
#[instruction(repo_name_padded: [u8;32])]
pub struct RepoBadge<'info> {
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
    )]
    pub dev_state: Account<'info, DevState>,

    /// CHECK: This is being verifed by the contraints
    #[account(
        mut,
        seeds = [b"dev_badge", dev.key().as_ref()],
        bump = dev_state.collection_asset_bump,
        constraint = !dev_badge.data_is_empty() @GhostErrors::CollectionNotInitialized,
    )]
    pub dev_badge: UncheckedAccount<'info>,

    #[account(
        init,
        payer = dev,
        space = RepoState::DISCRIMINATOR.len() + RepoState::INIT_SPACE,
        seeds = [b"repo_state", dev_badge.key().as_ref(), repo_name_padded.as_ref()],  // One repo state per dev per repo
        bump ,
    )]
    pub repo_state: Account<'info, RepoState>,

    /// CHECK: This will be checked and initialized by the core program
    #[account(
        mut,
        seeds = [b"repo_badge", dev_badge.key().as_ref(), repo_name_padded.as_ref()],     // One repo state per dev per repo
        bump,
    )]
    pub repo_badge: UncheckedAccount<'info>,

    /// CHECK: Sysvar instruction checked by address
    #[account(
        address = SYSVAR_INSTRUCTION_ID,
    )]
    pub instruction_sysvar: UncheckedAccount<'info>,

    /// CHECK: Metaplex core program
    #[account(
        address = CORE_PROGRAM_ID,
    )]
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> RepoBadge<'info> {
    pub fn mint_repo_badge(
        &mut self,
        repo_name_padded: [u8; 32],
        username_padded: [u8; 32],
        stars: u32,
        commits: u32,
        forks: u32,
        open_issues: u32,
        is_fork: u8,
        lang1: Vec<u8>,
        lang2: Vec<u8>,
        bumps: &RepoBadgeBumps,
    ) -> Result<()> {
        // Verify that the message( Repo stats ) is signed by the backend signer
        verify_signature(
            &self.instruction_sysvar.to_account_info(),
            &self.ghost_config.backend_pubkey,
        )?;

        // Minting repo badges for forked repo not allowed
        require!(is_fork != 1, GhostErrors::ForkedRepo);

        let config_seeds: &[&[&[u8]]] = &[&[b"ghost_config", &[self.ghost_config.bump]]];
        let repo_badge_seeds: &[&[&[u8]]] = &[&[
            b"repo_badge",
            &self.dev_badge.key().to_bytes(),
            repo_name_padded.as_ref(),
            &[bumps.repo_badge],
        ]];

        CreateV2CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.repo_badge.to_account_info())
            .payer(&self.dev.to_account_info())
            .collection(Some(&self.dev_badge.to_account_info()))
            .authority(Some(&self.ghost_config.to_account_info()))
            .system_program(&self.system_program.to_account_info())
            .name("AMM-Repo".to_string())
            .uri("https://ghostcheck/metadata/repo-image/dev".to_string())
            .plugins(vec![
                PluginAuthorityPair {
                    plugin: Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate {
                        frozen: true,
                    }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
                PluginAuthorityPair {
                    plugin: Plugin::UpdateDelegate(UpdateDelegate {
                        additional_delegates: vec![],
                    }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
                PluginAuthorityPair {
                    plugin: Plugin::Attributes(Attributes {
                        attribute_list: vec![
                            Attribute {
                                key: "repo_name".to_string(),
                                value: String::from_utf8(repo_name_padded.to_vec()).unwrap(),
                            },
                            Attribute {
                                key: "stars".to_string(),
                                value: stars.to_string(),
                            },
                            Attribute {
                                key: "commits".to_string(),
                                value: commits.to_string(),
                            },
                            Attribute {
                                key: "forks".to_string(),
                                value: forks.to_string(),
                            },
                        ],
                    }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
            ])
            .external_plugin_adapters(vec![])
            .invoke_signed(&[config_seeds[0], repo_badge_seeds[0]])?;

        // Update the parent collection state
        self.dev_state.verified_repos += 1;
        self.ghost_config.repo_badges_minted += 1;

        // Get current time
        let time_now = Clock::get()?.unix_timestamp;

        // Update the repo state
        self.repo_state.set_inner(RepoState {
            owner: self.dev.key(),
            dev_badge: self.dev_badge.key(),
            repo_name: repo_name_padded.to_vec(),
            hashed_username: username_padded,
            stars,
            commits,
            forks,
            open_issues,
            is_fork,
            lang1,
            lang2,
            last_updated: time_now,
            bump: bumps.repo_state,
            badge_bump: bumps.repo_badge,
        });

        Ok(())
    }
}
