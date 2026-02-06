use anchor_lang::prelude::*;
use sp1_solana::{verify_proof, GROTH16_VK_5_0_0_BYTES};

use crate::errors::GhostErrors;
use crate::state::{GhostConfig, UserAccount, VerificationReceipt};

#[derive(Accounts)]
#[instruction(public_inputs: [u8; 32])]
pub struct VerifyProof<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [b"ghost_config"],
        bump = ghost_config.bump,
    )]
    pub ghost_config: Account<'info, GhostConfig>,

    #[account(
        init,
        payer = user,
        space = VerificationReceipt::DISCRIMINATOR.len() + VerificationReceipt::INIT_SPACE,
        seeds = [b"receipt", public_inputs.as_ref()],
        bump,
    )]
    pub receipt: Account<'info, VerificationReceipt>,

    #[account(
        init,
        payer = user,
        space = UserAccount::DISCRIMINATOR.len() + UserAccount::INIT_SPACE,
        seeds = [b"user_account", user.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    pub system_program: Program<'info, System>,
}

pub fn process_verify_proof(
    ctx: Context<VerifyProof>,
    public_inputs: [u8; 32],
    zk_proof: Vec<u8>,
) -> Result<()> {
    msg!("Verifing Proof...");

    require!(
        public_inputs.len() == 64,
        GhostErrors::InvalidPublicInputLen
    );

    let vkey_as_str = std::str::from_utf8(&ctx.accounts.ghost_config.vkey_hash)
        .map_err(|_| GhostErrors::VKeyCastFailed)?;

    verify_proof(
        &zk_proof,
        &public_inputs,
        &vkey_as_str,
        &GROTH16_VK_5_0_0_BYTES,
    )
    .map_err(|_| GhostErrors::InvalidProof)?;

    let receipt = &mut ctx.accounts.receipt;
    receipt.user = ctx.accounts.user.key();
    receipt
        .hashed_username
        .copy_from_slice(&public_inputs[..32]);
    receipt.repo_hashed.copy_from_slice(&public_inputs[32..64]);
    receipt.is_minted = false;
    receipt.bump = ctx.bumps.receipt;

    ctx.accounts.user_account.set_inner(UserAccount {
        latest_receipt: ctx.accounts.receipt.key(),
        bump: ctx.bumps.user_account,
    });

    msg!("Verified !");

    Ok(())
}
