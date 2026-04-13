use anchor_lang::prelude::*;
mod storage;
mod error;
mod constants;

declare_id!("GgAUi3CiVHE8hxoAMu9pdXEJsrV2JHdfeqKgavdrXmdm");

#[program]
pub mod contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
