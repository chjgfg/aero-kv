use anchor_lang::prelude::*;

#[error_code]
pub enum Error {
    #[msg("invalid key, please input again")]
    InvalidKey,

    #[msg("invalid vlaue, please input again")]
    InvalidValue,

    #[msg("You are not authorized to modify this Key.")]
    Unauthorized,
}
