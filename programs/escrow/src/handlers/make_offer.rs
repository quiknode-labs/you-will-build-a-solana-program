use super::shared::transfer_tokens;
use crate::{error::ErrorCode, state::Offer};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

// See https://www.anchor-lang.com/docs/account-constraints#instruction-attribute
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct MakeOffer<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub offered_token: InterfaceAccount<'info, Mint>,

    #[account(mint::token_program = token_program)]
    pub wanted_token: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = offered_token,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_token_account_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = Offer::DISCRIMINATOR.len() + Offer::INIT_SPACE,
        seeds = [b"offer", id.to_le_bytes().as_ref()],
        bump
    )]
    pub offer: Account<'info, Offer>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = offered_token,
        associated_token::authority = offer,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

// Handle the make offer instruction by:
// 1. Moving the tokens from the maker's ATA to the vault
// 2. Saving the details of the offer to the offer account
pub fn make_offer(
    context: Context<MakeOffer>,
    id: u64,
    token_a_offered_amount: u64,
    wanted_amount: u64,
) -> Result<()> {
    // Validate amounts
    require!(token_a_offered_amount > 0, ErrorCode::InvalidAmount);
    require!(wanted_amount > 0, ErrorCode::InvalidAmount);

    // Validate token mints are different
    require!(
        context.accounts.offered_token.key() != context.accounts.wanted_token.key(),
        ErrorCode::InvalidTokenMint
    );

    // Move the tokens from the maker's ATA to the vault
    transfer_tokens(
        &context.accounts.maker_token_account_a,
        &context.accounts.vault,
        &token_a_offered_amount,
        &context.accounts.offered_token,
        &context.accounts.maker.to_account_info(),
        &context.accounts.token_program,
        None,
    )
    .map_err(|_| ErrorCode::InsufficientMakerBalance)?;

    // Save the details of the offer to the offer account
    context.accounts.offer.set_inner(Offer {
        id,
        maker: context.accounts.maker.key(),
        offered_token: context.accounts.offered_token.key(),
        wanted_token: context.accounts.wanted_token.key(),
        wanted_amount,
        bump: context.bumps.offer,
    });
    Ok(())
}
