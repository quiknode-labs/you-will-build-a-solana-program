use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use super::shared::{close_token_account, transfer_tokens};
use crate::state::Offer;

#[derive(Accounts)]
pub struct RefundOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    pub offered_token: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = offered_token,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = maker,
        seeds = [b"offer", offer.id.to_le_bytes().as_ref()],
        bump = offer.bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        mut,
        associated_token::mint = offered_token,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

// Handle the refund offer instruction by:
// 1. Returning the tokens from the vault to the maker's account
// 2. Closing the vault and returning the rent to the maker
pub fn refund_offer(context: Context<RefundOffer>) -> Result<()> {
    let offer_account_seeds = &[
        b"offer",
        &context.accounts.offer.id.to_le_bytes()[..],
        &[context.accounts.offer.bump],
    ];
    let signers_seeds = Some(&offer_account_seeds[..]);

    // Return the tokens from the vault to the maker's account
    transfer_tokens(
        &context.accounts.vault,
        &context.accounts.maker_token_account_a,
        &context.accounts.vault.amount,
        &context.accounts.offered_token,
        &context.accounts.offer.to_account_info(),
        &context.accounts.token_program,
        signers_seeds,
    )?;

    // Close the vault and return the rent to the maker
    close_token_account(
        &context.accounts.vault,
        &context.accounts.maker.to_account_info(),
        &context.accounts.offer.to_account_info(),
        &context.accounts.token_program,
        signers_seeds,
    )
}
