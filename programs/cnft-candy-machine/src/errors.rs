use anchor_lang::error_code;

#[error_code]
pub enum CustomError {
    #[msg("User Not Allowed")]
    UserNotAllowed,
    #[msg("User Already Claimed")]
    AlreadyClaimed,
    #[msg("Invalid Allow Mint")]
    InvalidAllowMint,
    #[msg("Candy Machine is not active")]
    CandyMachineInactive,
}