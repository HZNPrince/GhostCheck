use anchor_lang::prelude::*;

use crate::{errors::GhostErrors, program::GhostCheck, state::GhostConfig};

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

    pub system_program: Program<'info, System>,

    #[account(constraint = this_program.programdata_address()? == Some(program_data.key()) @GhostErrors::ProgramDataMismatch)]
    pub this_program: Program<'info, GhostCheck>,

    #[account(constraint = program_data.upgrade_authority_address == Some(admin.key()) @GhostErrors::UpgradeAuthorityMismatch)]
    pub program_data: Account<'info, ProgramData>,
}

impl<'info> InitConfig<'info> {
    pub fn init_config(&mut self, backend_pubkey: [u8; 32], bumps: &InitConfigBumps) -> Result<()> {
        self.ghost_config.set_inner(GhostConfig {
            admin: self.admin.key(),
            backend_pubkey,
            dev_badges_minted: 0,
            repo_badges_minted: 0,
            vouches_count: 0,
            bump: bumps.ghost_config,
        });

        Ok(())
    }
}
