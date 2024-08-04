use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;
use crate::state::{Config, TreeStatus};

use mpl_bubblegum::{instructions::CreateTreeConfigCpiBuilder, ID as BUBBLEGUM_ID};
use spl_account_compression::ID as SPL_ACCOUNT_COMPRESSION_ID;
use spl_noop::ID as SPL_NOOP_ID;

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
    /// CHECK:
    #[account(
        seeds = [b"authority", authority.key().as_ref()],
        bump,
    )]
    pub mint_authority: UncheckedAccount<'info>,
    /// CHECK:
    #[account(
        mut,
        seeds = [merkle_tree.key().as_ref()],
        bump,
        seeds::program = bubblegum_program.key()
    )]
    pub tree_authority: UncheckedAccount<'info>,
    /// CHECK:
    #[account(mut)]
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK:
    #[account(address = SPL_NOOP_ID)]
    pub log_wrapper: UncheckedAccount<'info>,
    /// CHECK:
    #[account(address = BUBBLEGUM_ID)]
    pub bubblegum_program: UncheckedAccount<'info>,
    /// CHECK:
    #[account(address = SPL_ACCOUNT_COMPRESSION_ID)]
    pub compression_program: UncheckedAccount<'info>,
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

    pub fn init_tree(&mut self, max_depth: u32, max_buffer_size: u32) -> Result<()> {
        let bubblegum_program = &self.bubblegum_program.to_account_info();
        let merkle_tree = &self.merkle_tree.to_account_info();
        let tree_creator = &self.tree_authority.to_account_info();
        let payer = &self.authority.to_account_info();
        let log_wrapper = &self.log_wrapper.to_account_info();
        let compression_program = &self.compression_program.to_account_info();


        CreateTreeConfigCpiBuilder::new(bubblegum_program)
            .max_buffer_size(max_buffer_size)
            .max_depth(max_depth)
            .merkle_tree(merkle_tree)
            .tree_config(tree_creator)
            .tree_creator(tree_creator)
            .payer(payer)
            .public(false)
            .log_wrapper(log_wrapper)
            .compression_program(compression_program)
            .invoke()?;          
        
        Ok(())
    }
}