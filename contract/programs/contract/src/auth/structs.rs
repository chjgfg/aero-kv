use anchor_lang::prelude::*;

#[account]
pub struct AuthConfig {
    pub admin: Pubkey,
    pub paused: bool,
}

impl AuthConfig {
    pub const LEN: usize = 8 + 32 + 1;
}
