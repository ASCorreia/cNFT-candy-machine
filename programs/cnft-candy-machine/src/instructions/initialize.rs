use anchor_lang::prelude::*;

use crate::state::{Config, TreeStatus};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        seeds = [b"config", authority.key().as_ref()],
        bump,
        space = Config::INIT_SPACE
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn init_config(&mut self, total_supply: u32) -> Result<()> {
        self.config.set_inner(
            Config {
                authority: self.authority.key(),
                allow_list: vec![],
                total_supply,
                current_supply: 0,
                status: TreeStatus::Inactive,
            },
        );
        Ok(())
    }

    pub fn init_tree(&mut self) -> Result<()> {
        
        Ok(())
    }
}