use crate::errors::GhostErrors;
use anchor_lang::prelude::{
    sysvar::instructions::{load_current_index_checked, load_instruction_at_checked},
    *,
};

pub fn verify_signature(instruction_sysvar: &AccountInfo, backend_pubkey: &[u8; 32]) -> Result<()> {
    let current_ix = load_current_index_checked(instruction_sysvar)?;
    let ix = load_instruction_at_checked((current_ix as usize) - 1, instruction_sysvar)?;

    let ed_25519_id: Pubkey =
        Pubkey::new_from_array(solana_program::ed25519_program::ID.to_bytes());

    require!(ix.program_id == ed_25519_id, GhostErrors::InvalidSignature);

    let data = ix.data;
    let pubkey_bytes: [u8; 32] = data[16..48]
        .try_into()
        .map_err(|_| GhostErrors::PubkeyParseFailed)?;

    require!(
        &pubkey_bytes == backend_pubkey,
        GhostErrors::BackendPubkeyMismatch
    );

    Ok(())
}
