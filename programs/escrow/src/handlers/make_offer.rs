use anchor_lang::prelude::*;

// See https://www.anchor-lang.com/docs/account-constraints#instruction-attribute
#[derive(Accounts)]
#[instruction(id: u64)]
pub struct MakeOffer {}

// Handle the make offer instruction by:
// 1. Moving the tokens from the maker's ATA to the vault
// 2. Saving the details of the offer to the offer account
pub fn make_offer(_context: Context<MakeOffer>) -> Result<()> {
    Ok(())
}
