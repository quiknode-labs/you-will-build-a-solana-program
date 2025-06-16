#![allow(unexpected_cfgs)]
// Stops Rust Analyzer complaining about missing configs
// See https://solana.stackexchange.com/questions/17777

use anchor_lang::prelude::*;
use handlers::*;

pub mod constants;
pub mod error;
pub mod handlers;
pub mod state;

#[cfg(test)]
mod test_helpers;

#[cfg(test)]
mod escrow_test_helpers;

declare_id!("8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y");

#[program]
pub mod escrow {
    use super::*;

    pub fn make_offer(context: Context<MakeOffer>) -> Result<()> {
        handlers::make_offer::make_offer(context)
    }

    pub fn take_offer(context: Context<TakeOffer>) -> Result<()> {
        handlers::take_offer::take_offer(context)
    }

    pub fn refund_offer(context: Context<RefundOffer>) -> Result<()> {
        handlers::refund_offer::refund_offer(context)
    }
}

#[cfg(test)]
mod tests;
