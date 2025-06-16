use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::ErrorCode, state::Offer};

use super::shared::{close_token_account, transfer_tokens};

#[derive(Accounts)]
pub struct TakeOffer<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    pub offered_token: InterfaceAccount<'info, Mint>,

    pub wanted_token: InterfaceAccount<'info, Mint>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = offered_token,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = wanted_token,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
    )]
    pub taker_token_account_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = taker,
        associated_token::mint = wanted_token,
        associated_token::authority = maker,
        associated_token::token_program = token_program,
    )]
    pub maker_token_account_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = wanted_token,
        seeds = [b"offer", offer.id.to_le_bytes().as_ref()],
        bump = offer.bump
    )]
    offer: Account<'info, Offer>,

    #[account(
        mut,
        associated_token::mint = offered_token,
        associated_token::authority = offer,
        associated_token::token_program = token_program,
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

// Handle the take offer instruction by:
// 1. Sending the wanted tokens from the taker to the maker
// 2. Withdrawing the offered tokens from the vault to the taker and closing the vault
pub fn take_offer(context: Context<TakeOffer>) -> Result<()> {
    // Send the wanted tokens from the taker to the maker
    transfer_tokens(
        &context.accounts.taker_token_account_b,
        &context.accounts.maker_token_account_b,
        &context.accounts.offer.wanted_amount,
        &context.accounts.wanted_token,
        &context.accounts.taker.to_account_info(),
        &context.accounts.token_program,
        None,
    )
    .map_err(|_| ErrorCode::InsufficientTakerBalance)?;

    let offer_account_seeds = &[
        b"offer",
        &context.accounts.offer.id.to_le_bytes()[..],
        &[context.accounts.offer.bump],
    ];
    let signers_seeds = Some(&offer_account_seeds[..]);

    // Withdraw the offered tokens from the vault to the taker
    transfer_tokens(
        &context.accounts.vault,
        &context.accounts.taker_token_account_a,
        &context.accounts.vault.amount,
        &context.accounts.offered_token,
        &context.accounts.offer.to_account_info(),
        &context.accounts.token_program,
        signers_seeds,
    )?;

    // Close the vault and return the rent to the maker
    close_token_account(
        &context.accounts.vault,
        &context.accounts.taker.to_account_info(),
        &context.accounts.offer.to_account_info(),
        &context.accounts.token_program,
        signers_seeds,
    )
}
