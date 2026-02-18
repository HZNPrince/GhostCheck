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
    #[msg("Repo provided for mint badge is forked")]
    ForkedRepo,
    #[msg("dev_state.dev_addr dosent match the signer passed")]
    IncorrectDevState,
    #[msg("Dev badge passed and addr found in repo_state mismatch")]
    DevBadgeMismatch,
    #[msg("Verifying dev failed due to reputation level lower than min_lvl passed")]
    DevVerificationFailed,
    #[msg("Vouchers level has not reached the specified threshold to vouch for others")]
    LvlNotReached,
    #[msg("Invalid min_lvl input should range between 1 to 5")]
    ReputationLvlInvalid,
    #[msg("Voucher tried to vouch for himself")]
    SelfVouchDenied,
}
