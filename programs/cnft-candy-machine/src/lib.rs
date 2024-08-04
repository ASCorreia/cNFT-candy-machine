use anchor_lang::prelude::*;

declare_id!("5FjsgRt8fkWv22pyksKcVBkieh1J15qEv7WfSP4CNLyx");

mod state;
mod instructions;
mod constants;

use instructions::*;

#[program]
pub mod cnft_candy_machine {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, total_supply: u32) -> Result<()> {
        ctx.accounts.init_config(total_supply)?;
        ctx.accounts.init_tree()
    }
}