use anchor_lang::prelude::*;

use crate::state::{
    AllowListStruct, 
    Config
};

#[derive(Accounts)]
pub struct AllowList<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    pub user: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"config", authority.key().as_ref()],
        bump = config.bump,
        realloc = Config::INIT_SPACE + (config.allow_list.len() * AllowListStruct::INIT_SPACE) + AllowListStruct::INIT_SPACE,
        realloc::payer = authority,
        realloc::zero = true,
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>,
}

impl<'info> AllowList<'info> {
    pub fn add(&mut self, amount: u8) -> Result<()> {
        self.config.allow_list.push(
            AllowListStruct{
                user: self.user.key(),
                amount,
            }
        );
        Ok(())
    }
}