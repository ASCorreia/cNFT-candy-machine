use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, metadata::Metadata, token::{Mint, Token, TokenAccount}};
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
    pub config: Box<Account<'info, Config>>,
    pub allow_mint: Option<Account<'info, Mint>>,
    #[account(
        init,
        payer = authority,
        seeds = [b"collection", config.key().as_ref()],
        bump,
        mint::decimals = 0,
        mint::authority = config,
        mint::freeze_authority = config,
    )]
    pub collection: Account<'info, Mint>,
    /// CHECK: This account is checked in the instruction
    #[account(mut)]
    pub tree_config: UncheckedAccount<'info>,
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
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
}

impl<'info> Initialize<'info> {
    pub fn init_config(&mut self, total_supply: u32, bumps: &InitializeBumps) -> Result<()> {
        let allow_mint = match self.allow_mint.clone() {
            Some(value) => Some(value.key()),
            None => None,
        };

        self.config.set_inner(
            Config {
                authority: self.authority.key(),
                allow_list: vec![],
                allow_mint,
                collection: self.collection.key(),
                total_supply,
                current_supply: 0,
                status: TreeStatus::Active,
                bump: bumps.config, 
            },
        );
        Ok(())
    }

    pub fn init_tree(&mut self, max_depth: u32, max_buffer_size: u32) -> Result<()> {
        let seeds = &[
            &b"config"[..], 
            &self.authority.key.as_ref(),
            &[self.config.bump],
        ];
        let signer_seeds = &[&seeds[..]];
        
        let bubblegum_program = &self.bubblegum_program.to_account_info();
        let tree_config = &self.tree_config.to_account_info();
        let merkle_tree = &self.merkle_tree.to_account_info();
        let tree_creator = &self.config.to_account_info();
        let payer = &self.authority.to_account_info();
        let log_wrapper = &self.log_wrapper.to_account_info();
        let compression_program = &self.compression_program.to_account_info();
        let system_program = &self.system_program.to_account_info();

        CreateTreeConfigCpiBuilder::new(bubblegum_program)
            .tree_config(tree_config)
            .merkle_tree(merkle_tree)
            .payer(payer)
            .tree_creator(tree_creator)
            .log_wrapper(log_wrapper)
            .compression_program(compression_program)
            .system_program(system_program)
            .max_depth(max_depth)
            .max_buffer_size(max_buffer_size)
            .public(false)
            .invoke_signed(signer_seeds)?;          
        
        Ok(())
    }
}