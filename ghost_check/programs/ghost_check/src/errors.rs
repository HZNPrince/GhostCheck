use anchor_lang::prelude::*;

#[error_code]
pub enum GhostErrors {
    #[msg("Program Id and Program Data Id Mismatch")]
    PgIdPgDataMismatch,
    #[msg("admin key and upgrade authority key Mismatch")]
    UpgradeAuthorityMismatch,
    #[msg("DevBadge asset passed is already initialized")]
    CollectionAlreadyInitialized,
    #[msg("Collection(Dev_badge) for dev not initialized")]
    CollectionNotInitialized,
    #[msg("Program Data does not match the program data passed")]
    ProgramDataMismatch,
    #[msg("Signature Verification Failed")]
    InvalidSignature,
    #[msg("Failed to parse [16..48] to pubkey_bytes of instruction 0")]
    PubkeyParseFailed,
    #[msg("Backend Pubkey of signingkey dosent match the pubkey signed in instruction 0")]
    BackendPubkeyMismatch,
    #[msg("Ed25519Program id dosent match the ix program id passed at instruction 0")]
    Ed25519PrgmIdMismatch,
}
