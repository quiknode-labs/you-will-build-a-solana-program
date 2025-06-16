use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct MakeOffer {
    // MakeOffer (in capitals) is a struct of names accounts that the
    // make_offer() function will use.
}

// Handle the make offer instruction by:
// 1. Moving the tokens from the maker's ATA to the vault
// 2. Saving the details of the offer to the offer account
pub fn make_offer(_context: Context<MakeOffer>) -> Result<()> {
    Ok(())
}
