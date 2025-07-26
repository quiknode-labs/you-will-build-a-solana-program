use litesvm::LiteSVM;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_kite::{
    create_associated_token_account, create_token_mint, deploy_program, get_pda_and_bump,
    mint_tokens_to_account, send_transaction_from_instructions, SolanaKiteError,
};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use std::cell::Cell;
use std::str::FromStr;

use crate::generated::instructions::{
    MakeOfferBuilder, MakeOfferInstructionArgs, RefundOfferBuilder, TakeOfferBuilder,
};

pub const PROGRAM_ID: &str = "8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y";

/// Standard token unit for token A (1 token = 1_000_000_000 lamports for 9 decimals)
pub const TOKEN_A: u64 = 1_000_000_000;

/// Standard token unit for token B (1 token = 1_000_000_000 lamports for 9 decimals)
pub const TOKEN_B: u64 = 1_000_000_000;

/// Complete escrow test environment containing all necessary components for testing
///
/// This struct holds all the accounts, keypairs, and state needed for comprehensive
/// escrow testing scenarios. It's returned by `setup_escrow_test()` and provides
/// a convenient way to access all test components.
pub struct EscrowTestEnvironment {
    /// The LiteSVM instance for simulating Solana transactions
    pub litesvm: LiteSVM,
    /// The escrow program ID
    pub program_id: Pubkey,
    /// The mint authority that can create and mint tokens
    pub _mint_authority: Keypair,
    /// Token mint A (the first token in escrow trades)
    pub token_mint_a: Keypair,
    /// Token mint B (the second token in escrow trades)
    pub token_mint_b: Keypair,
    /// Alice's keypair (typically the offer maker)
    pub alice: Keypair,
    /// Bob's keypair (typically the offer taker)
    pub bob: Keypair,
    /// Alice's token account for token A
    pub alice_token_account_a: Pubkey,
    /// Alice's token account for token B
    pub alice_token_account_b: Pubkey,
    /// Bob's token account for token A
    pub bob_token_account_a: Pubkey,
    /// Bob's token account for token B
    pub bob_token_account_b: Pubkey,
}

/// Sets up a complete escrow test environment with all necessary components
///
/// This function performs the following setup steps:
/// 1. Creates a new LiteSVM instance for transaction simulation
/// 2. Deploys the escrow program to the test environment
/// 3. Creates a mint authority and funds it with SOL
/// 4. Creates two token mints (A and B) with 9 decimals
/// 5. Creates Alice and Bob keypairs and funds them with SOL
/// 6. Creates associated token accounts for both users and both token types
/// 7. Mints initial token balances:
///    - Alice: 10 token A, 0 token B
///    - Bob: 0 token A, 5 token B
///
/// # Returns
///
/// Returns an `EscrowTestEnvironment` struct containing all the test components
/// needed for escrow testing scenarios.
///
/// # Example
///
/// ```rust
/// let env = setup_escrow_test();
///
/// // Create an offer using Alice
/// let offer_id = 12345u64;
/// let (offer_account, _) = Pubkey::find_program_address(
///     &[b"offer", &offer_id.to_le_bytes()],
///     &env.program_id
/// );
///
/// // Use the environment components for testing
/// let make_offer_accounts = MakeOfferAccounts {
///     associated_token_program: spl_associated_token_account::ID,
///     token_program: spl_token::ID,
///     system_program: anchor_lang::system_program::ID,
///     maker: env.alice.pubkey(),
///     token_mint_a: env.token_mint_a.pubkey(),
///     token_mint_b: env.token_mint_b.pubkey(),
///     maker_token_account_a: env.alice_token_account_a,
///     offer_account,
///     vault: spl_associated_token_account::get_associated_token_address(&offer_account, &env.token_mint_a.pubkey()),
/// };
/// ```
pub fn setup_escrow_test() -> EscrowTestEnvironment {
    let mut litesvm = LiteSVM::new();
    let program_id = get_program_id();

    // Deploy the escrow program
    deploy_program(&mut litesvm, &program_id, "../../target/deploy/escrow.so").unwrap();

    // Create and fund mint authority
    let mint_authority = Keypair::new();
    litesvm
        .airdrop(&mint_authority.pubkey(), 1_000_000_000)
        .unwrap();

    // Create token mints
    let token_mint_a = create_token_mint(&mut litesvm, &mint_authority, 9).unwrap();
    let token_mint_b = create_token_mint(&mut litesvm, &mint_authority, 9).unwrap();

    // Create and fund Alice and Bob
    let alice = Keypair::new();
    let bob = Keypair::new();
    litesvm.airdrop(&alice.pubkey(), 1_000_000_000).unwrap();
    litesvm.airdrop(&bob.pubkey(), 1_000_000_000).unwrap();

    // Create associated token accounts for both users and both token types
    let alice_token_account_a = create_associated_token_account(
        &mut litesvm,
        &alice,
        &token_mint_a.pubkey(),
        &mint_authority,
    )
    .unwrap();
    let alice_token_account_b = create_associated_token_account(
        &mut litesvm,
        &alice,
        &token_mint_b.pubkey(),
        &mint_authority,
    )
    .unwrap();
    let bob_token_account_a = create_associated_token_account(
        &mut litesvm,
        &bob,
        &token_mint_a.pubkey(),
        &mint_authority,
    )
    .unwrap();
    let bob_token_account_b = create_associated_token_account(
        &mut litesvm,
        &bob,
        &token_mint_b.pubkey(),
        &mint_authority,
    )
    .unwrap();

    // Mint initial token balances
    // Alice: 10 token A, 0 token B
    // Bob: 0 token A, 5 token B
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_a.pubkey(),
        &alice_token_account_a,
        10 * TOKEN_A, // Alice gets 10 token A
        &mint_authority,
    )
    .unwrap();
    mint_tokens_to_account(
        &mut litesvm,
        &token_mint_b.pubkey(),
        &bob_token_account_b,
        5 * TOKEN_B, // Bob gets 5 token B
        &mint_authority,
    )
    .unwrap();

    EscrowTestEnvironment {
        litesvm,
        program_id,
        _mint_authority: mint_authority,
        token_mint_a,
        token_mint_b,
        alice,
        bob,
        alice_token_account_a,
        alice_token_account_b,
        bob_token_account_a,
        bob_token_account_b,
    }
}

pub fn get_program_id() -> Pubkey {
    Pubkey::from_str(PROGRAM_ID).unwrap()
}

thread_local! {
    static OFFER_ID_COUNTER: Cell<u64> = Cell::new(1);
}

/// Generates a unique offer ID for testing
///
/// This function returns incrementing offer IDs starting from 1, ensuring
/// each test gets unique IDs to avoid conflicts between test cases.
pub fn generate_offer_id() -> u64 {
    OFFER_ID_COUNTER.with(|counter| {
        let id = counter.get();
        counter.set(id + 1);
        id
    })
}

/// Helper function to create MakeOfferAccounts with standard program IDs
///
/// This function eliminates the repetitive initialization of the three standard
/// program IDs (associated_token_program, token_program, system_program) that
/// are always the same constants across all tests. Instead of copy-pasting
/// these three lines in every test, this helper focuses on the variable fields.

pub struct MakeOfferAccounts {
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub maker_token_account_a: Pubkey,
    pub offer_account: Pubkey,
    pub vault: Pubkey,
}

pub fn build_make_offer_instruction(
    accounts: MakeOfferAccounts,
    args: MakeOfferInstructionArgs,
) -> Instruction {
    let mut make_offer_builder = MakeOfferBuilder::new();

    // Construct accounts
    make_offer_builder
        .maker(accounts.maker)
        .token_mint_a(accounts.token_mint_a)
        .token_mint_b(accounts.token_mint_b)
        .maker_token_account_a(accounts.maker_token_account_a)
        .offer(accounts.offer_account)
        .vault(accounts.vault);

    // Construct args
    make_offer_builder
        .id(args.id)
        .token_a_offered_amount(args.token_a_offered_amount)
        .token_b_wanted_amount(args.token_b_wanted_amount);

    // Build instruction
    make_offer_builder.instruction()
}

pub struct TakeOfferAccounts {
    pub taker: Pubkey,
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub taker_token_account_a: Pubkey,
    pub taker_token_account_b: Pubkey,
    pub maker_token_account_b: Pubkey,
    pub offer_account: Pubkey,
    pub vault: Pubkey,
}

pub fn build_take_offer_instruction(accounts: TakeOfferAccounts) -> Instruction {
    let mut take_offer_builder = TakeOfferBuilder::new();

    // Construct accounts
    take_offer_builder
        .taker(accounts.taker)
        .maker(accounts.maker)
        .token_mint_a(accounts.token_mint_a)
        .token_mint_b(accounts.token_mint_b)
        .taker_token_account_a(accounts.taker_token_account_a)
        .taker_token_account_b(accounts.taker_token_account_b)
        .maker_token_account_b(accounts.maker_token_account_b)
        .offer(accounts.offer_account)
        .vault(accounts.vault);

    // Build instruction
    take_offer_builder.instruction()
}

pub struct RefundOfferAccounts {
    pub maker: Pubkey,
    pub token_mint_a: Pubkey,
    pub maker_token_account_a: Pubkey,
    pub offer_account: Pubkey,
    pub vault: Pubkey,
}

pub fn build_refund_offer_instruction(accounts: RefundOfferAccounts) -> Instruction {
    let mut refund_builder = RefundOfferBuilder::new();

    // Construct accounts
    refund_builder
        .maker(accounts.maker)
        .token_mint_a(accounts.token_mint_a)
        .maker_token_account_a(accounts.maker_token_account_a)
        .offer(accounts.offer_account)
        .vault(accounts.vault);

    // Build instruction
    refund_builder.instruction()
}

/// Executes a complete make_offer flow: creates PDAs, builds accounts, and executes instruction
///
/// This helper eliminates the repetitive pattern of creating offer_account and vault PDAs,
/// building MakeOfferAccounts, and executing the make_offer instruction that appears in
/// multiple tests.
pub fn execute_make_offer(
    test_env: &mut EscrowTestEnvironment,
    offer_id: u64,
    maker: &Keypair,
    maker_token_account_a: Pubkey,
    token_a_offered_amount: u64,
    token_b_wanted_amount: u64,
) -> Result<(Pubkey, Pubkey), SolanaKiteError> {
    // Create PDAs
    let (offer_account, _offer_bump) = get_pda_and_bump(
        &[
            b"offer".as_ref().into(),
            offer_id.to_le_bytes().as_ref().into(),
        ],
        &test_env.program_id,
    );
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &test_env.token_mint_a.pubkey(),
    );

    let make_offer_accounts = MakeOfferAccounts {
        maker: maker.pubkey(),
        token_mint_a: test_env.token_mint_a.pubkey(),
        token_mint_b: test_env.token_mint_b.pubkey(),
        maker_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_args = MakeOfferInstructionArgs {
        id: offer_id,
        token_a_offered_amount,
        token_b_wanted_amount,
    };

    // Build and execute instruction
    let make_offer_instruction = build_make_offer_instruction(make_offer_accounts, make_offer_args);

    send_transaction_from_instructions(
        &mut test_env.litesvm,
        vec![make_offer_instruction],
        &[maker],
        &maker.pubkey(),
    )?;

    Ok((offer_account, vault))
}

/// Executes a complete take_offer flow: builds accounts and executes instruction
pub fn execute_take_offer(
    test_env: &mut EscrowTestEnvironment,
    taker: &Keypair,
    maker: &Keypair,
    taker_token_account_a: Pubkey,
    taker_token_account_b: Pubkey,
    maker_token_account_b: Pubkey,
    offer_account: Pubkey,
    vault: Pubkey,
) -> Result<(), SolanaKiteError> {
    let take_offer_accounts = TakeOfferAccounts {
        taker: taker.pubkey(),
        maker: maker.pubkey(),
        token_mint_a: test_env.token_mint_a.pubkey(),
        token_mint_b: test_env.token_mint_b.pubkey(),
        taker_token_account_a,
        taker_token_account_b,
        maker_token_account_b,
        offer_account,
        vault,
    };

    let take_offer_instruction = build_take_offer_instruction(take_offer_accounts);

    send_transaction_from_instructions(
        &mut test_env.litesvm,
        vec![take_offer_instruction],
        &[taker],
        &taker.pubkey(),
    )
}

/// Executes a complete refund_offer flow: builds accounts and executes instruction
pub fn execute_refund_offer(
    test_env: &mut EscrowTestEnvironment,
    maker: &Keypair,
    maker_token_account_a: Pubkey,
    offer_account: Pubkey,
    vault: Pubkey,
) -> Result<(), SolanaKiteError> {
    let refund_offer_accounts = RefundOfferAccounts {
        maker: maker.pubkey(),
        token_mint_a: test_env.token_mint_a.pubkey(),
        maker_token_account_a,
        offer_account,
        vault,
    };

    let refund_instruction = build_refund_offer_instruction(refund_offer_accounts);

    send_transaction_from_instructions(
        &mut test_env.litesvm,
        vec![refund_instruction],
        &[maker],
        &maker.pubkey(),
    )
}
