use anchor_lang::prelude::*;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::metadata::{
    MasterEditionAccount, 
    Metadata, 
    MetadataAccount
};
use anchor_spl::token::{
    burn, 
    Burn, 
    Mint, 
    Token, TokenAccount
};
use mpl_bubblegum::instructions::MintToCollectionV1CpiBuilder;
use mpl_bubblegum::types::{
    Collection, 
    MetadataArgs, 
    TokenProgramVersion, 
    TokenStandard
};
use mpl_bubblegum::ID as BUBBLEGUM_ID;
use spl_account_compression::ID as SPL_ACCOUNT_COMPRESSION_ID;
use spl_noop::ID as SPL_NOOP_ID;

use crate::state::TreeStatus;
use crate::{
    state::Config, 
    CustomError
};

#[derive(Accounts)]
pub struct MintNFT<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub authority: SystemAccount<'info>,
    #[account(
        mut,
        seeds = [b"config", authority.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub allow_mint: Option<Account<'info, Mint>>,
    #[account(mut)]
    pub allow_mint_ata: Option<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [b"collection", config.key().as_ref()],
        bump,
    )]
    pub collection: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            collection.key().as_ref()
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub collection_metadata: Account<'info, MetadataAccount>,
    #[account(
        mut,
        seeds = [
            b"metadata",
            metadata_program.key().as_ref(),
            collection.key().as_ref(),
            b"edition",
        ],
        seeds::program = metadata_program.key(),
        bump,
    )]
    pub collection_edition: Account<'info, MasterEditionAccount>,
    #[account(
        init,
        payer = user,
        associated_token::mint = collection,
        associated_token::authority = user,
    )]
    pub destination: Box<Account<'info, TokenAccount>>,
    /// CHECK: Tree Config account that will be checked by the Bubblegum Program
    #[account(mut)]
    pub tree_config: UncheckedAccount<'info>,
    pub leaf_owner: SystemAccount<'info>,
    /// CHECK: Merkle Tree account that will be checked by the Bubblegum Program
    #[account(mut)]
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK: SPL NOOP Program checked by the corresponding address
    #[account(address = SPL_NOOP_ID)]
    pub log_wrapper: UncheckedAccount<'info>,
    /// CHECK: Bubblegum Program checked by the corresponding address
    #[account(address = BUBBLEGUM_ID)]
    pub bubblegum_program: UncheckedAccount<'info>,
    /// CHECK: SPL Account Compression Program checked by the corresponding address
    #[account(address = SPL_ACCOUNT_COMPRESSION_ID)]
    pub compression_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub metadata_program: Program<'info, Metadata>,
}

impl<'info> MintNFT<'info> {
    pub fn mint_cnft(&mut self, name: String, symbol: String, uri: String) -> Result<()> {

        require!(self.config.status != TreeStatus::Inactive, CustomError::CandyMachineInactive);

        if self.config.status != TreeStatus::Public {
            if let (Some(allow_mint), Some(allow_mint_ata)) = (&self.allow_mint, &self.allow_mint_ata) {

                require!(allow_mint.key() == self.config.allow_mint.unwrap(), CustomError::InvalidAllowMint);

                let cpi_program = self.token_program.to_account_info();

                let cpi_accounts = Burn {
                    mint: allow_mint.to_account_info(),
                    from: allow_mint_ata.to_account_info(),
                    authority: self.user.to_account_info(),
                };

                let cpi_context = CpiContext::new(cpi_program, cpi_accounts);

                burn(cpi_context, 1 * 10_u8.pow(allow_mint.decimals as u32) as u64)?;
            }
            else {
                self.config.allow_list.iter().find(|x| x.user == self.user.key()).ok_or(CustomError::UserNotAllowed)?;

                let user_struct = self.config.allow_list.iter_mut().find(|x| x.user == self.user.key()).unwrap();
            
                if user_struct.amount == 0 {
                    return Err(CustomError::AlreadyClaimed.into());
                }
                user_struct.amount -= 1;    
            }
        }

        let seeds = &[
            &b"config"[..], 
            &self.authority.key.as_ref(),
            &[self.config.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        MintToCollectionV1CpiBuilder::new(&self.bubblegum_program.to_account_info())
            .tree_config(&self.tree_config.to_account_info())
            .leaf_owner(&self.leaf_owner.to_account_info())
            .leaf_delegate(&self.leaf_owner)
            .merkle_tree(&self.merkle_tree.to_account_info())
            .payer(&self.user.to_account_info())
            .tree_creator_or_delegate(&self.config.to_account_info())
            .collection_authority(&self.config.to_account_info())
            .collection_authority_record_pda(None)
            .collection_mint(&self.collection.to_account_info())
            .collection_metadata(&self.collection_metadata.to_account_info())
            .collection_edition(&self.collection_edition.to_account_info())
            .bubblegum_signer(&self.config.to_account_info())
            .log_wrapper(&self.log_wrapper.to_account_info())
            .compression_program(&self.compression_program.to_account_info())
            .token_metadata_program(&self.metadata_program.to_account_info())
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
                    collection: Some(Collection {
                        verified: true,
                        key: self.collection.key(),
                    }),
                    token_program_version: TokenProgramVersion::Original,
                    token_standard: Some(TokenStandard::NonFungible),
                }
            )
        .invoke_signed(signer_seeds)?;

        if self.config.current_supply == self.config.total_supply {
            self.close_account()?;
        }

        Ok(())
    }

    pub fn close_account(&mut self) -> Result<()> {

        **self.authority.lamports.borrow_mut() = self.authority.lamports().checked_add(self.config.to_account_info().lamports()).unwrap();

        **self.config.to_account_info().lamports.borrow_mut() = 0;
        
        Ok(())
    }
}