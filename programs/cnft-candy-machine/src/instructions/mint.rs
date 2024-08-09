use anchor_lang::prelude::*;

use mpl_bubblegum::instructions::MintV1CpiBuilder;
use mpl_bubblegum::types::{MetadataArgs, TokenProgramVersion, TokenStandard};
use mpl_bubblegum::ID as BUBBLEGUM_ID;
use spl_account_compression::ID as SPL_ACCOUNT_COMPRESSION_ID;
use spl_noop::ID as SPL_NOOP_ID;

use crate::{state::Config, CustomError};

#[derive(Accounts)]
pub struct Mint<'info> {
    pub user: Signer<'info>,
    pub authority: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"config", authority.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,
    /// CHECK:
    #[account(mut)]
    pub tree_config: UncheckedAccount<'info>,
    pub leaf_owner: SystemAccount<'info>,
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

impl<'info> Mint<'info> {
    pub fn mint_cnft(&mut self, name: String, symbol: String, uri: String) -> Result<()> {
        let seeds = &[
            &b"config"[..], 
            &self.authority.key.as_ref(),
            &[self.config.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        self.config.allow_list.iter().find(|x| x.user == self.user.key()).ok_or(CustomError::UserNotAllowed)?;

        let user_struct = self.config.allow_list.iter_mut().find(|x| x.user == self.user.key()).unwrap();
        
        if user_struct.amount == 0 {
            return Err(CustomError::AlreadyClaimed.into());
        }
        user_struct.amount -= 1;

        MintV1CpiBuilder::new(&self.bubblegum_program.to_account_info())
        .tree_config(&self.tree_config.to_account_info())
        .leaf_owner(&self.leaf_owner.to_account_info())
        .leaf_delegate(&self.leaf_owner)
        .merkle_tree(&mut self.merkle_tree.to_account_info())
        .payer(&self.user.to_account_info())
        .tree_creator_or_delegate(&self.config.to_account_info())
        .log_wrapper(&self.log_wrapper.to_account_info())
        .compression_program(&self.compression_program.to_account_info())
        .system_program(&self.system_program.to_account_info())
        .metadata(
            MetadataArgs {
                name,
                symbol,
                uri,
                creators: vec![],
                seller_fee_basis_points: 0,
                primary_sale_happened: false,
                is_mutable: false,
                edition_nonce: Some(0),
                uses: None,
                collection: None,
                token_program_version: TokenProgramVersion::Original,
                token_standard: Some(TokenStandard::NonFungible),
            }
        )
        .invoke_signed(signer_seeds)?;

        self.config.current_supply += 1;
        
        Ok(())
    }
}