use spl_program_error::*;

#[spl_program_error]
pub enum ProtocolError {
    #[error("Incorrect soulbound mint")]
    IncorrectSoulboundMint,
    #[error("Incorrect soulbound token account")]
    IncorrectSoulboundTokenAccount,
    #[error("Soulbound token account has tokens")]
    SoulboundTokenAccountHasTokens,
    #[error("Soulbound token account is empty")]
    SoulboundTokenAccountIsEmpty,
    #[error("Profile already initialized")]
    ProfileAlreadyInitialized,
    #[error("Profile not initialized")]
    ProfileNotInitialized,
    #[error("Username too long")]
    UsernameTooLong,
    #[error("Incorrect validation account was provided")]
    IncorrectValidationAccount,
}
