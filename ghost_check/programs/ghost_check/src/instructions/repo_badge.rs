use crate::errors::GhostErrors;
use crate::state::{DevState, GhostConfig, RepoState};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::{
    load_instruction_at_checked, ID as SYSVAR_INSTRUCTION_ID,
};
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
        seeds = [b"dev_badge_collection", dev.key().as_ref()],
        bump = dev_state.collection_asset_bump,
        constraint = !dev_badge.data_is_empty() @GhostErrors::CollectionNotInitialized,
    )]
    pub dev_badge: UncheckedAccount<'info>,

    #[account(
        init,
        payer = dev,
        space = RepoState::DISCRIMINATOR.len() + RepoState::INIT_SPACE,
        seeds = [b"repo_state", dev_badge.key().as_ref()],
        bump ,
    )]
    pub repo_state: Account<'info, RepoState>,

    /// CHECK: This will be checked and initialized by the core program
    #[account(
        mut,
        seeds = [b"repo_badge_asset", dev_badge.key().as_ref(), repo_name_padded.as_ref()],
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
    pub fn verify_signature(&self) -> Result<()> {
        let ix = load_instruction_at_checked(0, &self.instruction_sysvar)?;

        // Check that ix program id matches the ed25519 program id
        let ed25519_id: Pubkey =
            Pubkey::new_from_array(solana_program::ed25519_program::ID.to_bytes());

        require!(
            ix.program_id == ed25519_id,
            GhostErrors::Ed25519PrgmIdMismatch
        );

        // Check that ix data at position of pubkey of signer matches the backend_pubkey
        let ix_data = ix.data;
        let ix_public_key: [u8; 32] = ix_data[16..48]
            .try_into()
            .map_err(|_| GhostErrors::PubkeyParseFailed)?;

        require!(
            ix_public_key == self.ghost_config.backend_pubkey,
            GhostErrors::BackendPubkeyMismatch
        );

        Ok(())
    }

    pub fn mint_repo_badge(
        &mut self,
        repo_name_padded: [u8; 32],
        username_padded: [u8; 32],
        stars: u32,
        commits: u32,
        lang1: Vec<u8>,
        lang2: Vec<u8>,
        bumps: &RepoBadgeBumps,
    ) -> Result<()> {
        let config_seeds: &[&[&[u8]]] = &[&[b"ghost_config", &[self.ghost_config.bump]]];
        let repo_badge_seeds: &[&[&[u8]]] = &[&[
            b"repo_badge_asset",
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
                        ],
                    }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
            ])
            .external_plugin_adapters(vec![])
            .invoke_signed(&[config_seeds[0], repo_badge_seeds[0]])?;

        // Update the parent collection state
        self.dev_state.verified_repo += 1;

        // Update the repo state
        self.repo_state.set_inner(RepoState {
            owner: self.dev.key(),
            repo_name: repo_name_padded.to_vec(),
            dev_badge: self.dev_badge.key(),
            hashed_username: username_padded,
            stars,
            commits,
            lang1,
            lang2,
        });

        Ok(())
    }
}
