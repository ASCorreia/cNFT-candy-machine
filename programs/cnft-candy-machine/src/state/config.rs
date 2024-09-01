use anchor_lang::prelude::*;

use crate::constants::{ANCHOR_DESCRIMINATOR_SIZE, PUBKEY_SIZE, TREE_STATUS_SIZE, U32_SIZE, VEC_PREFIX_SIZE};

#[account]
pub struct Config {
    pub authority: Pubkey,
    pub allow_list: Vec<AllowListStruct>,
    pub allow_mint: Option<Pubkey>,
    pub collection: Pubkey,
    pub total_supply: u32,
    pub current_supply: u32,
    pub status: TreeStatus,
    pub bump: u8,
}

impl Space for Config {
    const INIT_SPACE: usize = ANCHOR_DESCRIMINATOR_SIZE + PUBKEY_SIZE + VEC_PREFIX_SIZE + (1 + PUBKEY_SIZE) + PUBKEY_SIZE + (U32_SIZE * 2) + TREE_STATUS_SIZE + 1; 
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum TreeStatus {
    Inactive,
    Active,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, InitSpace)]
pub struct AllowListStruct {
    pub user: Pubkey,
    pub amount: u8,
}