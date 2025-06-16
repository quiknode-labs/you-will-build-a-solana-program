use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct TakeOffer {}

// Handle the take offer instruction by:
// 1. Sending the wanted tokens from the taker to the maker
// 2. Withdrawing the offered tokens from the vault to the taker and closing the vault
pub fn take_offer(_context: Context<TakeOffer>) -> Result<()> {
    Ok(())
}
