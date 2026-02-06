use anchor_lang::prelude::*;

#[error_code]
pub enum GhostErrors {
    #[msg("Program Id and Program Data Id Mismatch")]
    PgIdPgDataMismatch,
    #[msg("admin key and upgrade authority key Mismatch")]
    UpgradeAuthorityMismatch,
    #[msg("invalid public input parameter length passed ")]
    InvalidPublicInputLen,
    #[msg("Proof verification failed")]
    InvalidProof,
    #[msg("Could not cast vkey from [u8;64] to str")]
    VKeyCastFailed,
}
