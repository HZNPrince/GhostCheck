use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct GhostConfig {
    pub admin: Pubkey,
    pub vkey_hash: [u8; 64],
    pub collection_mint: Pubkey,
    pub nft_minted: u32,
    pub bump: u8,
}

#[derive(InitSpace)]
#[account]
pub struct UserAccount {
    pub latest_receipt: Pubkey,
    pub bump: u8,
}

#[derive(InitSpace)]
#[account]
pub struct VerificationReceipt {
    pub user: Pubkey,
    pub hashed_username: [u8; 32],
    pub repo_hashed: [u8; 32],
    pub is_minted: bool,
    pub bump: u8,
}
