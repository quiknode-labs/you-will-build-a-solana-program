use solana_signer::Signer;

mod helpers;

use helpers::generated::instructions::MakeOfferInstructionArgs;
use helpers::*;

use solana_kite::{
    assert_token_balance, check_account_is_closed, get_pda_and_bump, seeds,
    send_transaction_from_instructions,
};

#[test]
fn test_make_offer_succeeds() {
    let mut test_environment = setup_escrow_test();

    let offer_id = generate_offer_id();
    let (offer_account, _offer_bump) =
        get_pda_and_bump(&seeds!["offer", offer_id], &test_environment.program_id);
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &test_environment.token_mint_a.pubkey(),
    );

    let make_offer_accounts = MakeOfferAccounts {
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        maker_token_account_a: test_environment.alice_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_args = MakeOfferInstructionArgs {
        id: offer_id,
        token_a_offered_amount: 1 * TOKEN_A,
        token_b_wanted_amount: 1 * TOKEN_B,
    };

    let make_offer_instruction = build_make_offer_instruction(make_offer_accounts, make_offer_args);

    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![make_offer_instruction],
        &[&test_environment.alice],
        &test_environment.alice.pubkey(),
    );

    assert!(result.is_ok(), "Valid offer should succeed");
}

#[test]
fn test_duplicate_offer_id_fails() {
    let mut test_environment = setup_escrow_test();

    let offer_id = generate_offer_id();
    let (offer_account, _offer_bump) =
        get_pda_and_bump(&seeds!["offer", offer_id], &test_environment.program_id);
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &test_environment.token_mint_a.pubkey(),
    );

    let make_offer_accounts = MakeOfferAccounts {
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        maker_token_account_a: test_environment.alice_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_args = MakeOfferInstructionArgs {
        id: offer_id,
        token_a_offered_amount: 1 * TOKEN_A,
        token_b_wanted_amount: 1 * TOKEN_B,
    };

    let make_offer_instruction = build_make_offer_instruction(make_offer_accounts, make_offer_args);

    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![make_offer_instruction],
        &[&test_environment.alice],
        &test_environment.alice.pubkey(),
    );
    assert!(result.is_ok(), "First offer should succeed");

    let make_offer_accounts_with_existing_offer_id = MakeOfferAccounts {
        maker: test_environment.bob.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        maker_token_account_a: test_environment.bob_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_args_with_existing_offer_id = MakeOfferInstructionArgs {
        id: offer_id,
        token_a_offered_amount: 1 * TOKEN_A,
        token_b_wanted_amount: 1 * TOKEN_B,
    };

    let make_offer_instruction_with_existing_offer_id = build_make_offer_instruction(
        make_offer_accounts_with_existing_offer_id,
        make_offer_args_with_existing_offer_id,
    );

    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![make_offer_instruction_with_existing_offer_id],
        &[&test_environment.bob],
        &test_environment.bob.pubkey(),
    );
    assert!(result.is_err(), "Second offer with same ID should fail");
}

#[test]
fn test_insufficient_funds_fails() {
    let mut test_environment = setup_escrow_test();

    // Try to create offer with more tokens than Alice owns
    let offer_id = generate_offer_id();
    let (offer_account, _offer_bump) =
        get_pda_and_bump(&seeds!["offer", offer_id], &test_environment.program_id);
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &test_environment.token_mint_a.pubkey(),
    );

    let make_offer_accounts = MakeOfferAccounts {
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        maker_token_account_a: test_environment.alice_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_args = MakeOfferInstructionArgs {
        id: offer_id,
        token_a_offered_amount: 1000 * TOKEN_A, // Try to offer 1000 tokens (Alice only has 10)
        token_b_wanted_amount: 1 * TOKEN_B,
    };

    let make_offer_instruction = build_make_offer_instruction(make_offer_accounts, make_offer_args);

    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![make_offer_instruction],
        &[&test_environment.alice],
        &test_environment.alice.pubkey(),
    );
    assert!(result.is_err(), "Offer with insufficient funds should fail");
}

#[test]
fn test_same_token_mints_fails() {
    let mut test_environment = setup_escrow_test();

    // Try to create offer with same token mint for both token_a and token_b
    let offer_id = generate_offer_id();
    let (offer_account, _offer_bump) =
        get_pda_and_bump(&seeds!["offer", offer_id], &test_environment.program_id);
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &test_environment.token_mint_a.pubkey(),
    );

    let make_offer_accounts = MakeOfferAccounts {
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_a.pubkey(), // Same mint for both
        maker_token_account_a: test_environment.alice_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_args = MakeOfferInstructionArgs {
        id: offer_id,
        token_a_offered_amount: 1 * TOKEN_A,
        token_b_wanted_amount: 1 * TOKEN_B,
    };

    let make_offer_instruction = build_make_offer_instruction(make_offer_accounts, make_offer_args);

    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![make_offer_instruction],
        &[&test_environment.alice],
        &test_environment.alice.pubkey(),
    );
    assert!(result.is_err(), "Offer with same token mints should fail");
}

#[test]
fn test_zero_token_b_wanted_amount_fails() {
    let mut test_environment = setup_escrow_test();

    // Try to create offer with zero token_b_wanted_amount
    let offer_id = generate_offer_id();
    let (offer_account, _offer_bump) =
        get_pda_and_bump(&seeds!["offer", offer_id], &test_environment.program_id);
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &test_environment.token_mint_a.pubkey(),
    );

    let make_offer_accounts = MakeOfferAccounts {
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        maker_token_account_a: test_environment.alice_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_args = MakeOfferInstructionArgs {
        id: offer_id,
        token_a_offered_amount: 1 * TOKEN_A,
        token_b_wanted_amount: 0, // Zero wanted amount
    };

    let make_offer_instruction = build_make_offer_instruction(make_offer_accounts, make_offer_args);

    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![make_offer_instruction],
        &[&test_environment.alice],
        &test_environment.alice.pubkey(),
    );
    assert!(
        result.is_err(),
        "Offer with zero token_b_wanted_amount should fail"
    );
}

#[test]
fn test_zero_token_a_offered_amount_fails() {
    let mut test_environment = setup_escrow_test();

    // Try to create offer with zero token_a_offered_amount
    let offer_id = generate_offer_id();
    let (offer_account, _offer_bump) =
        get_pda_and_bump(&seeds!["offer", offer_id], &test_environment.program_id);
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &test_environment.token_mint_a.pubkey(),
    );

    let make_offer_accounts = MakeOfferAccounts {
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        maker_token_account_a: test_environment.alice_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_args = MakeOfferInstructionArgs {
        id: offer_id,
        token_a_offered_amount: 0, // Zero offered amount
        token_b_wanted_amount: 1 * TOKEN_B,
    };

    let make_offer_instruction = build_make_offer_instruction(make_offer_accounts, make_offer_args);

    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![make_offer_instruction],
        &[&test_environment.alice],
        &test_environment.alice.pubkey(),
    );
    assert!(
        result.is_err(),
        "Offer with zero token_a_offered_amount should fail"
    );
}

#[test]
fn test_take_offer_success() {
    let mut test_environment = setup_escrow_test();

    // Alice creates an offer: 3 token A for 2 token B
    let offer_id = generate_offer_id();
    let alice = test_environment.alice.insecure_clone();
    let alice_token_account_a = test_environment.alice_token_account_a;
    let (offer_account, vault) = execute_make_offer(
        &mut test_environment,
        offer_id,
        &alice,
        alice_token_account_a,
        3 * TOKEN_A,
        2 * TOKEN_B,
    )
    .unwrap();

    // Bob takes the offer
    let bob = test_environment.bob.insecure_clone();
    let bob_token_account_a = test_environment.bob_token_account_a;
    let bob_token_account_b = test_environment.bob_token_account_b;
    let alice_token_account_b = test_environment.alice_token_account_b;
    execute_take_offer(
        &mut test_environment,
        &bob,
        &alice,
        bob_token_account_a,
        bob_token_account_b,
        alice_token_account_b,
        offer_account,
        vault,
    )
    .unwrap();

    // Check balances
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.alice_token_account_a,
        7 * TOKEN_A,
        "Alice should have 7 token A left",
    );
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.alice_token_account_b,
        2 * TOKEN_B,
        "Alice should have received 2 token B",
    );
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.bob_token_account_a,
        3 * TOKEN_A,
        "Bob should have received 3 token A",
    );
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.bob_token_account_b,
        3 * TOKEN_B,
        "Bob should have 3 token B left",
    );

    // Check that the offer account is closed after being taken
    check_account_is_closed(
        &test_environment.litesvm,
        &offer_account,
        "Offer account should be closed after being taken",
    );
}

#[test]
fn test_refund_offer_success() {
    let mut test_environment = setup_escrow_test();

    // Alice creates an offer: 3 token A for 2 token B
    let offer_id = generate_offer_id();
    let alice = test_environment.alice.insecure_clone();
    let alice_token_account_a = test_environment.alice_token_account_a;
    let (offer_account, vault) = execute_make_offer(
        &mut test_environment,
        offer_id,
        &alice,
        alice_token_account_a,
        3 * TOKEN_A,
        2 * TOKEN_B,
    )
    .unwrap();

    // Check that Alice's balance decreased after creating the offer
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.alice_token_account_a,
        7 * TOKEN_A,
        "Alice should have 7 token A left after creating offer",
    );

    // Alice refunds the offer
    execute_refund_offer(
        &mut test_environment,
        &alice,
        alice_token_account_a,
        offer_account,
        vault,
    )
    .unwrap();

    // Check that Alice's balance is restored after refunding
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.alice_token_account_a,
        10 * TOKEN_A,
        "Alice should have all 10 token A back after refunding",
    );

    // Check that the offer account is closed
    check_account_is_closed(
        &test_environment.litesvm,
        &offer_account,
        "Offer account should be closed after refund",
    );
}

#[test]
fn test_non_maker_cannot_refund_offer() {
    let mut test_environment = setup_escrow_test();

    // Alice creates an offer: 3 token A for 2 token B
    let offer_id = generate_offer_id();
    let (offer_account, _offer_bump) =
        get_pda_and_bump(&seeds!["offer", offer_id], &test_environment.program_id);
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &test_environment.token_mint_a.pubkey(),
    );

    let make_offer_accounts = MakeOfferAccounts {
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        maker_token_account_a: test_environment.alice_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_args = MakeOfferInstructionArgs {
        id: offer_id,
        token_a_offered_amount: 3 * TOKEN_A,
        token_b_wanted_amount: 2 * TOKEN_B,
    };

    let make_offer_instruction = build_make_offer_instruction(make_offer_accounts, make_offer_args);

    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![make_offer_instruction],
        &[&test_environment.alice],
        &test_environment.alice.pubkey(),
    );
    assert!(result.is_ok(), "Alice's offer should succeed");

    // Bob tries to refund Alice's offer (should fail)
    let refund_offer_accounts = RefundOfferAccounts {
        maker: test_environment.bob.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        maker_token_account_a: test_environment.alice_token_account_a,
        offer_account,
        vault,
    };

    let refund_instruction = build_refund_offer_instruction(refund_offer_accounts);
    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![refund_instruction],
        &[&test_environment.bob],
        &test_environment.bob.pubkey(),
    );
    assert!(
        result.is_err(),
        "Non-maker should not be able to refund an offer"
    );

    // Verify that Alice's balance is still the same (offer not refunded)
    assert_token_balance(
        &test_environment.litesvm,
        &test_environment.alice_token_account_a,
        7 * TOKEN_A,
        "Alice's balance should remain unchanged after failed refund attempt",
    );

    // Verify that the offer account still exists (invert the check)
    let offer_account_data = test_environment.litesvm.get_account(&offer_account);
    assert!(
        offer_account_data.is_some() && !offer_account_data.unwrap().data.is_empty(),
        "Offer account should still exist after failed refund attempt"
    );
}

#[test]
fn test_take_offer_insufficient_funds_fails() {
    let mut test_environment = setup_escrow_test();

    // Create an offer from Alice for a large amount of token B
    let large_token_b_amount = 1000 * TOKEN_B; // Much larger than Bob's balance (he has 5)
    let offer_id = generate_offer_id();
    let (offer_account, _offer_bump) =
        get_pda_and_bump(&seeds!["offer", offer_id], &test_environment.program_id);
    let vault = spl_associated_token_account::get_associated_token_address(
        &offer_account,
        &test_environment.token_mint_a.pubkey(),
    );

    let make_offer_accounts = MakeOfferAccounts {
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        maker_token_account_a: test_environment.alice_token_account_a,
        offer_account,
        vault,
    };

    let make_offer_args = MakeOfferInstructionArgs {
        id: offer_id,
        token_a_offered_amount: 1 * TOKEN_A,
        token_b_wanted_amount: large_token_b_amount,
    };

    let make_offer_instruction = build_make_offer_instruction(make_offer_accounts, make_offer_args);

    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![make_offer_instruction],
        &[&test_environment.alice],
        &test_environment.alice.pubkey(),
    );
    assert!(result.is_ok(), "Alice's offer should succeed");

    // Try to take the offer with Bob who has insufficient token B
    let take_offer_accounts = TakeOfferAccounts {
        taker: test_environment.bob.pubkey(),
        maker: test_environment.alice.pubkey(),
        token_mint_a: test_environment.token_mint_a.pubkey(),
        token_mint_b: test_environment.token_mint_b.pubkey(),
        taker_token_account_a: test_environment.bob_token_account_a,
        taker_token_account_b: test_environment.bob_token_account_b,
        maker_token_account_b: test_environment.alice_token_account_b,
        offer_account,
        vault,
    };

    let take_offer_instruction = build_take_offer_instruction(take_offer_accounts);
    let result = send_transaction_from_instructions(
        &mut test_environment.litesvm,
        vec![take_offer_instruction],
        &[&test_environment.bob],
        &test_environment.bob.pubkey(),
    );
    assert!(
        result.is_err(),
        "Take offer with insufficient funds should fail"
    );
}
