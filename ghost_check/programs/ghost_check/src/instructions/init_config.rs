use crate::{errors::GhostErrors, program::GhostCheck, state::GhostConfig};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct InitConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = GhostConfig::DISCRIMINATOR.len() + GhostConfig::INIT_SPACE,
        seeds = [b"ghost_config"],
        bump,
    )]
    pub ghost_config: Account<'info, GhostConfig>,

    #[account(
        constraint = this_program.programdata_address()? == Some(program_data.key() ) @ GhostErrors::PgIdPgDataMismatch,
    )]
    pub this_program: Program<'info, GhostCheck>,

    #[account(
        constraint = program_data.upgrade_authority_address == Some(admin.key()) @GhostErrors::UpgradeAuthorityMismatch,
    )]
    pub program_data: Account<'info, ProgramData>,

    pub system_program: Program<'info, System>,
}

pub fn process_init_config(ctx: Context<InitConfig>, vkey_hash: [u8; 64]) -> Result<()> {
    ctx.accounts.ghost_config.set_inner(GhostConfig {
        admin: ctx.accounts.admin.key(),
        vkey_hash,
        collection_mint: Pubkey::default(),
        nft_minted: 0,
        bump: ctx.bumps.ghost_config,
    });

    Ok(())
}
