use anchor_lang::prelude::*;

#[error_code]
pub enum Error {
    #[msg("invalid key, please input again")]
    InvalidKey,
    #[msg("invalid vlaue, please input again")]
    InvalidValue,
    #[msg("You are not authorized to modify this Key.")]
    Unauthorized,
    #[msg("Key too long")]
    KeyTooLong,
    #[msg("Value too large")]
    ValueTooLarge,
    #[msg("Not found")]
    NotFound,
    #[msg("Invalid remaining accounts")]
    InvalidRemaining,
    #[msg("insufficient fee")]
    InsufficientFee,
    #[msg("System paused")]
    Paused,
}
