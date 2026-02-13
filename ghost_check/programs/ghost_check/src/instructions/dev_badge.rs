use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions::load_instruction_at_checked;
use mpl_core::{
    instructions::CreateCollectionV2CpiBuilder,
    types::{
        PermanentFreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair, UpdateDelegate,
    },
    ID as CORE_PROGRAM_ID,
};

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
    pub dev_badge_account: Account<'info, DevState>,

    /// CHECK: Core will create this
    #[account(mut,
        seeds = [b"dev_badge_collection", dev.key().as_ref()],
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
    pub fn verify_signature(&self) -> Result<()> {
        let ix = load_instruction_at_checked(0, &self.instruction_sysvar.to_account_info())?;

        let ed_25519_id: Pubkey =
            Pubkey::new_from_array(solana_program::ed25519_program::ID.to_bytes());

        require!(ix.program_id == ed_25519_id, GhostErrors::InvalidSignature);

        let data = ix.data;
        let pubkey_bytes: [u8; 32] = data[16..48]
            .try_into()
            .map_err(|_| GhostErrors::PubkeyParseFailed)?;

        require!(
            pubkey_bytes == self.ghost_config.backend_pubkey,
            GhostErrors::BackendPubkeyMismatch
        );

        Ok(())
    }

    pub fn mint_collection(
        &mut self,
        username: &[u8; 32],
        repo_count: u32,
        total_commits: u32,
        bumps: &DevBadgeBumps,
    ) -> Result<()> {
        // Create Collection Asset for new Dev
        let config_seeds: &[&[&[u8]]] = &[&[b"ghost_config", &[self.ghost_config.bump]]];
        let asset_seeds: &[&[&[u8]]] = &[&[
            b"dev_badge_collection",
            &self.dev.key().to_bytes(),
            &[bumps.asset],
        ]];

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
            ])
            .external_plugin_adapters(vec![])
            .invoke_signed(&[config_seeds[0], asset_seeds[0]])?;

        self.dev_badge_account.set_inner(DevState {
            address: self.asset.key(),
            hashed_username: username.clone(),
            total_commits,
            repo_count,
            verified_repo: 0,
            bump: bumps.dev_badge_account,
            collection_asset_bump: bumps.asset,
        });
        Ok(())
    }
}
