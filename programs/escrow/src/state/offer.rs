use anchor_lang::prelude::*;

// Stores details of an offer to swap token a for token b
// InitSpace allows us to calculate the space needed for this data
#[account]
pub struct Offer {
    // Details of the offer made, e.g. what who made it and what they want in return.
}
