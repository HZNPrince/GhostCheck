use anchor_lang::prelude::*;
use mpl_core::{
    instructions::CreateCollectionV2CpiBuilder,
    types::{
        Attribute, Attributes, PermanentFreezeDelegate, Plugin, PluginAuthority,
        PluginAuthorityPair, UpdateDelegate,
    },
    ID as CORE_PROGRAM_ID,
};

use crate::verify_signature;
use crate::{
    errors::GhostErrors,
    state::{DevState, GhostConfig},
};

#[derive(Accounts)]
pub struct DevBadge<'info> {
    #[account(mut)]
    pub dev: Signer<'info>,

    #[account(
        mut,
        seeds = [b"ghost_config"],
        bump = ghost_config.bump,
    )]
    pub ghost_config: Account<'info, GhostConfig>,

    #[account(
        init,
        payer = dev,
        space = DevState::DISCRIMINATOR.len() + DevState::INIT_SPACE,
        seeds = [b"dev_state", dev.key().as_ref()],
        bump,
    )]
    pub dev_state: Account<'info, DevState>,

    /// CHECK: Core will create this
    #[account(
        mut,
        seeds = [b"dev_badge", dev.key().as_ref()],
        bump,
        constraint = asset.data_is_empty() @GhostErrors::CollectionAlreadyInitialized)]
    pub asset: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: Needed for instruction introspection
    #[account(
        address = anchor_lang::solana_program::sysvar::instructions::ID,
    )]
    pub instruction_sysvar: UncheckedAccount<'info>,

    /// CHECK: Metaplex Core Program
    #[account(address = CORE_PROGRAM_ID)]
    pub core_program: UncheckedAccount<'info>,
}

impl<'info> DevBadge<'info> {
    pub fn mint_collection(
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
        bumps: &DevBadgeBumps,
    ) -> Result<()> {
        // Verify that the message(dev stats) is signed by the backend signer
        verify_signature(
            &self.instruction_sysvar.to_account_info(),
            &self.ghost_config.backend_pubkey,
        )?;

        // Create Collection Asset for new Dev
        let config_seeds: &[&[&[u8]]] = &[&[b"ghost_config", &[self.ghost_config.bump]]];
        let asset_seeds: &[&[&[u8]]] =
            &[&[b"dev_badge", &self.dev.key().to_bytes(), &[bumps.asset]]];

        CreateCollectionV2CpiBuilder::new(&self.core_program.to_account_info())
            .collection(&self.asset.to_account_info())
            .payer(&self.dev.to_account_info())
            .update_authority(Some(&self.ghost_config.to_account_info()))
            .system_program(&self.system_program)
            .name("Dev_Badge".to_string())
            .uri("https://GhostCheck/metadata/dev".to_string())
            .plugins(vec![
                PluginAuthorityPair {
                    plugin: Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate {
                        frozen: true,
                    }),
                    authority: Some(PluginAuthority::Address {
                        address: self.ghost_config.key(),
                    }),
                },
                PluginAuthorityPair {
                    plugin: Plugin::UpdateDelegate(UpdateDelegate {
                        additional_delegates: vec![],
                    }),
                    authority: Some(PluginAuthority::Address {
                        address: self.ghost_config.key(),
                    }),
                },
                PluginAuthorityPair {
                    plugin: Plugin::Attributes(Attributes {
                        attribute_list: vec![Attribute {
                            key: String::from("Dev"),
                            value: self.dev.key().to_string(),
                        }],
                    }),
                    authority: Some(PluginAuthority::UpdateAuthority),
                },
            ])
            .external_plugin_adapters(vec![])
            .invoke_signed(&[config_seeds[0], asset_seeds[0]])?;

        self.ghost_config.dev_badges_minted += 1;

        let current_time = Clock::get()?.unix_timestamp;

        self.dev_state.set_inner(DevState {
            dev_addr: self.dev.key(),
            asset_address: self.asset.key(),
            hashed_username: username.clone(),
            repo_count,
            owned_repo_count,
            total_stars,
            total_commits,
            prs_merged,
            issues_closed,
            followers,
            account_age_days,
            reputation_level,
            verified_repos: 0,
            vouch_count: 0,
            last_updated: current_time,
            bump: bumps.dev_state,
            collection_asset_bump: bumps.asset,
        });
        Ok(())
    }
}
