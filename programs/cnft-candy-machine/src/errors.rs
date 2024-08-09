use anchor_lang::error_code;

#[error_code]
pub enum CustomError {
    #[msg("User Not Allowed")]
    UserNotAllowed,
    #[msg("User Already Claimed")]
    AlreadyClaimed,
}