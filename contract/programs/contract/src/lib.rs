use anchor_lang::prelude::*;
mod auth;
mod constants;
mod error;
mod fee;
mod storage;
mod utils;

declare_id!("GgAUi3CiVHE8hxoAMu9pdXEJsrV2JHdfeqKgavdrXmdm");

#[program]
pub mod contract {
    use super::*;

    pub use super::storage::*;

    pub fn init_storage(ctx: Context<Initialize>) -> Result<()> {
        storage::init_storage(ctx)
    }

    pub fn get(ctx: Context<Get>, key: Vec<u8>) -> Result<Vec<u8>> {
        storage::get(ctx, key)
    }

    pub fn delete(ctx: Context<Delete>, key: Vec<u8>) -> Result<()> {
        storage::delete(ctx, key)
    }

    pub fn scan(ctx: Context<Scan>, start: Vec<u8>, limit: u64) -> Result<()> {
        storage::scan(ctx, start, limit)
    }

    pub fn upsert(ctx: Context<Upsert>, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        storage::upsert(ctx, key, value)
    }

    // ---------------------------------------------------------------------------------------------------------
    pub use super::auth::*;

    pub fn init_auth(ctx: Context<InitAuth>) -> Result<()> {
        auth::init_auth(ctx)
    }

    // ---------------------------------------------------------------------------------------------------------
    pub use super::fee::*;

    pub fn init_fee(ctx: Context<InitFee>) -> Result<()> {
        fee::init_fee(ctx)
    }
}
