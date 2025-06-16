use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RefundOffer {
    // RefundOffer (in capitals) is a struct of names accounts that the
    // refund_offer() function will use.
}

// Handle the refund offer instruction by:
// 1. Returning the tokens from the vault to the maker's account
// 2. Closing the vault and returning the rent to the maker
pub fn refund_offer(_context: Context<RefundOffer>) -> Result<()> {
    Ok(())
}
