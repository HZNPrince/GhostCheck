use anchor_lang::prelude::*;

use crate::errors::GhostErrors;
use crate::state::DevState;

#[derive(Accounts)]
#[instruction(dev_addr: [u8;32])]
pub struct VerifyDev<'info> {
    #[account(mut)]
    pub verifier: Signer<'info>,

    #[account(
        seeds = [b"dev_state", &dev_addr],
        bump,
    )]
    pub target_dev_state: Account<'info, DevState>,
}

impl<'info> VerifyDev<'info> {
    pub fn verify_dev(&self, dev_addr: [u8; 32], min_lvl: u8) -> Result<()> {
        // validate the min_lvl input
        require!(
            min_lvl >= 1 && min_lvl <= 5,
            GhostErrors::ReputationLvlInvalid
        );
        // Validate Dev
        require!(
            self.target_dev_state.reputation_level >= min_lvl,
            GhostErrors::DevVerificationFailed
        );

        Ok(())
    }
}
