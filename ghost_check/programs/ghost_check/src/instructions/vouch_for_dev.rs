use anchor_lang::prelude::*;

use crate::{
    errors::GhostErrors,
    state::{DevState, GhostConfig, VouchRecord},
};

#[derive(Accounts)]
#[instruction(target_addr: [u8;32])]
pub struct Vouch<'info> {
    #[account(mut)]
    pub voucher: Signer<'info>,

    #[account(
        mut,
        seeds= [b"ghost_config"],
        bump,
    )]
    pub ghost_config: Account<'info, GhostConfig>,

    #[account(
        seeds = [b"dev_state", voucher.key().as_ref()],
        bump,
        constraint = voucher_dev_state.dev_addr == voucher.key() @GhostErrors::IncorrectDevState,
        constraint = voucher_dev_state.reputation_level >= 2 @GhostErrors::LvlNotReached,
    )]
    pub voucher_dev_state: Account<'info, DevState>,

    #[account(
        mut,
        seeds = [b"dev_state", &target_addr],
        bump,
    )]
    pub target_dev_state: Account<'info, DevState>,

    #[account(
        init,
        payer = voucher,
        space = VouchRecord::DISCRIMINATOR.len() + VouchRecord::INIT_SPACE,
        seeds = [b"vouch_record", voucher.key().as_ref(), &target_addr],
        bump,
    )]
    pub vouch_record: Account<'info, VouchRecord>,

    pub system_program: Program<'info, System>,
}

impl<'info> Vouch<'info> {
    pub fn vouch_for_dev(&mut self, target_addr: [u8; 32], bumps: &VouchBumps) -> Result<()> {
        require!(
            self.voucher.key() != Pubkey::from(target_addr),
            GhostErrors::SelfVouchDenied
        );

        let time_now = Clock::get()?.unix_timestamp;

        // Create vouch record (PDA prevents duplicate vouches)
        self.vouch_record.set_inner(VouchRecord {
            voucher: self.voucher.key(),
            voucher_level: self.voucher_dev_state.reputation_level,
            target: Pubkey::from(target_addr),
            timestamp: time_now,
            bump: bumps.vouch_record,
        });

        // Update config and vouched dev state
        self.ghost_config.vouches_count += 1;
        self.target_dev_state.vouch_count += 1;

        Ok(())
    }
}
