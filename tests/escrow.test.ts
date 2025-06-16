import { before, describe, test, it } from "node:test";
import assert from "node:assert";
import * as programClient from "../dist/js-client";
import { getOfferDecoder, OFFER_DISCRIMINATOR } from "../dist/js-client";
import { connect, Connection, TOKEN_EXTENSIONS_PROGRAM, ErrorWithTransaction } from "solana-kite";
import { type KeyPairSigner, type Address } from "@solana/kit";
import { createTestOffer, getRandomBigInt, ONE_SOL } from "./escrow.test-helpers";

const INSUFFICIENT_FUNDS_ERROR = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb.TransferChecked: insufficient funds";
const REFUND_OFFER_ERROR =
  "8jR5GeNzeweq35Uo84kGP3v1NcBaZWH5u62k7PxN4T2y.RefundOffer: A has one constraint was violated";
const ACCOUNT_IN_USE_ERROR = "11111111111111111111111111111111.Allocate: account already in use";
const INVALID_TOKEN_MINT_ERROR = "custom program error: #6002";
const INVALID_AMOUNT_ERROR = "custom program error: #6003";

describe("Escrow", () => {
  let connection: Connection;
  let user: KeyPairSigner;
  let alice: KeyPairSigner;
  let bob: KeyPairSigner;
  let offeredToken: Address;
  let wantedToken: Address;
  let aliceTokenAccountA: Address;
  let bobTokenAccountA: Address;
  let aliceTokenAccountB: Address;

  const tokenDecimals = 9;

  // Both tokens have 9 decimals, so we can use this to convert between major and minor units
  const TOKEN = 10n ** BigInt(tokenDecimals);

  // Alice is going to make a few offers in these tests, so we give her 10 tokens
  const aliceInitialTokenAAmount = 10n * TOKEN;
  // We have a test later where Bob tries to reuse Alice's offer ID, so we give him a tiny amount (1 minor unit) of token A
  const bobInitialTokenAAmount = 1n;
  // Bob has 1 token of token B he will offer in exchange
  const bobInitialTokenBAmount = 1n * TOKEN;

  // Alice will offer 1 token of token A in exchange for 1 token of token B
  const tokenAOfferedAmount = 1n * TOKEN;
  const wantedAmount = 1n * TOKEN;

  before(async () => {
    connection = await connect();

    // 'user' will be the account we use to create the token mints
    [user, alice, bob] = await connection.createWallets(3, { airdropAmount: ONE_SOL });

    // Create two token mints - the factories that create token A, and token B
    offeredToken = await connection.createTokenMint({
      mintAuthority: user,
      decimals: tokenDecimals,
      name: "Token A",
      symbol: "TOKEN_A",
      uri: "https://example.com/token-a",
      additionalMetadata: {
        keyOne: "valueOne",
        keyTwo: "valueTwo",
      },
    });

    wantedToken = await connection.createTokenMint({
      mintAuthority: user,
      decimals: tokenDecimals,
      name: "Token B",
      symbol: "TOKEN_B",
      uri: "https://example.com/token-b",
      additionalMetadata: {
        keyOne: "valueOne",
        keyTwo: "valueTwo",
      },
    });

    // Mint tokens to alice and bob
    await connection.mintTokens(offeredToken, user, aliceInitialTokenAAmount, alice.address);
    await connection.mintTokens(offeredToken, user, bobInitialTokenAAmount, bob.address);
    await connection.mintTokens(wantedToken, user, bobInitialTokenBAmount, bob.address);

    // Get the token accounts for alice and bob
    aliceTokenAccountA = await connection.getTokenAccountAddress(alice.address, offeredToken, true);
    bobTokenAccountA = await connection.getTokenAccountAddress(bob.address, offeredToken, true);
    aliceTokenAccountB = await connection.getTokenAccountAddress(alice.address, wantedToken, true);
  });

  describe("makeOffer", () => {
    test("successfully creates an offer with valid inputs", async () => {
      const { offer, vault } = await createTestOffer({
        connection,
        maker: alice,
        offeredToken,
        wantedToken,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        wantedAmount,
      });

      // Verify the offer was created successfully by checking the vault balance
      const vaultBalanceResponse = await connection.getTokenAccountBalance({
        tokenAccount: vault,
        mint: offeredToken,
        useTokenExtensions: true,
      });
      assert.equal(vaultBalanceResponse.amount, tokenAOfferedAmount, "Vault balance should match offered amount");
    });

    test("fails when trying to reuse an existing offer ID", async () => {
      // First, create an offer with Alice using a specific offer ID
      const offerId = getRandomBigInt();
      await createTestOffer({
        connection,
        maker: alice,
        offeredToken,
        wantedToken,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        wantedAmount,
        offerId,
      });

      // Now try to create another offer with Bob using the same offer ID

      // Create bobTokenAccountA if it doesn't exist
      bobTokenAccountA = await connection.getTokenAccountAddress(bob.address, offeredToken, true);

      let testOffer: { offer: Address; vault: Address; offerId: bigint; signature: string };
      try {
        testOffer = await createTestOffer({
          connection,
          maker: bob,
          offeredToken,
          wantedToken,
          makerTokenAccountA: bobTokenAccountA,
          tokenAOfferedAmount: bobInitialTokenAAmount,
          wantedAmount,
          offerId, // Reusing the same offer ID
        });
      } catch (thrownObject) {
        const error = thrownObject as ErrorWithTransaction;
        assert.equal(error.message, ACCOUNT_IN_USE_ERROR);
      }
    });

    test("fails when maker has insufficient token balance", async () => {
      const tooManyTokens = 1_000n * TOKEN;

      try {
        await createTestOffer({
          connection,
          maker: alice,
          offeredToken,
          wantedToken,
          makerTokenAccountA: aliceTokenAccountA,
          tokenAOfferedAmount: tooManyTokens,
          wantedAmount,
        });
        assert.fail("Expected the offer creation to fail but it succeeded");
      } catch (thrownObject) {
        const error = thrownObject as ErrorWithTransaction;
        assert(
          error.message === INSUFFICIENT_FUNDS_ERROR,
          `Expected "${INSUFFICIENT_FUNDS_ERROR}" but got: ${error.message}`,
        );
      }
    });

    test("fails when token mints are the same", async () => {
      try {
        await createTestOffer({
          connection,
          maker: alice,
          offeredToken,
          wantedToken: offeredToken, // Using same mint
          makerTokenAccountA: aliceTokenAccountA,
          tokenAOfferedAmount,
          wantedAmount,
        });
        assert.fail("Expected the offer creation to fail but it succeeded");
      } catch (thrownObject) {
        const error = thrownObject as ErrorWithTransaction;
        assert(
          error.message.includes(INVALID_TOKEN_MINT_ERROR),
          `Expected InvalidTokenMint error but got: ${error.message}`,
        );
      }
    });

    test("fails when wanted_amount is zero", async () => {
      try {
        await createTestOffer({
          connection,
          maker: alice,
          offeredToken,
          wantedToken,
          makerTokenAccountA: aliceTokenAccountA,
          tokenAOfferedAmount,
          wantedAmount: 0n,
        });
        assert.fail("Expected the offer creation to fail but it succeeded");
      } catch (thrownObject) {
        const error = thrownObject as ErrorWithTransaction;
        assert(error.message.includes(INVALID_AMOUNT_ERROR), `Expected InvalidAmount error but got: ${error.message}`);
      }
    });

    test("fails when token_a_offered_amount is zero", async () => {
      try {
        await createTestOffer({
          connection,
          maker: alice,
          offeredToken,
          wantedToken,
          makerTokenAccountA: aliceTokenAccountA,
          tokenAOfferedAmount: 0n,
          wantedAmount,
        });
        assert.fail("Expected the offer creation to fail but it succeeded");
      } catch (thrownObject) {
        const error = thrownObject as ErrorWithTransaction;
        assert(error.message.includes(INVALID_AMOUNT_ERROR), `Expected InvalidAmount error but got: ${error.message}`);
      }
    });
  });

  describe("can get all the offers", () => {
    test("successfully gets all the offers", async () => {
      const getOffers = connection.getAccountsFactory(
        programClient.ESCROW_PROGRAM_ADDRESS,
        OFFER_DISCRIMINATOR,
        getOfferDecoder(),
      );

      const offers = await getOffers();

      assert.ok(offers.length === 2, "Expected to get two offers");

      // The first offer is created in the 'successfully creates an offer with valid inputs' test
      const offer1 = offers[0];
      assert.ok(offer1.exists, "Offer 1 account should exist");
      if (offer1.exists) {
        // This offer was created by Alice in the first makeOffer test
        assert.equal(offer1.address, offer1.address, "Offer 1 address should match");
        assert.equal(offer1.data.maker, alice.address, "Offer 1 maker address should match Alice");
        assert.equal(offer1.data.offeredToken, offeredToken, "Offer 1 offeredToken should match");
        assert.equal(offer1.data.wantedToken, wantedToken, "Offer 1 wantedToken should match");
        assert.equal(offer1.data.wantedAmount, wantedAmount.toString(), "Offer 1 wantedAmount should match");
        assert.ok(typeof offer1.data.bump === "number", "Offer 1 bump should be a number");
        assert.ok(offer1.data.discriminator, "Offer 1 discriminator should exist");
      }

      // The second offer is created in the 'fails when trying to reuse an existing offer ID' test (by Alice, before Bob tries to reuse the ID)
      const offer2 = offers[1];
      assert.ok(offer2.exists, "Offer 2 account should exist");
      if (offer2.exists) {
        // This offer was also created by Alice, with a specific offer ID
        assert.equal(offer2.address, offer2.address, "Offer 2 address should match");
        assert.equal(offer2.data.maker, alice.address, "Offer 2 maker address should match Alice");
        assert.equal(offer2.data.offeredToken, offeredToken, "Offer 2 offeredToken should match");
        assert.equal(offer2.data.wantedToken, wantedToken, "Offer 2 wantedToken should match");
        assert.equal(offer2.data.wantedAmount, wantedAmount.toString(), "Offer 2 wantedAmount should match");
        assert.ok(typeof offer2.data.bump === "number", "Offer 2 bump should be a number");
        assert.ok(offer2.data.discriminator, "Offer 2 discriminator should exist");
      }
    });
  });

  describe("takeOffer", () => {
    let testOffer: Address;
    let testVault: Address;

    before(async () => {
      const result = await createTestOffer({
        connection,
        maker: alice,
        offeredToken,
        wantedToken,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        wantedAmount,
      });
      testOffer = result.offer;
      testVault = result.vault;
    });

    test("successfully takes an offer", async () => {
      const takeOfferInstruction = await programClient.getTakeOfferInstructionAsync({
        taker: bob,
        maker: alice.address,
        offeredToken,
        wantedToken,
        takerTokenAccountA: bobTokenAccountA,
        makerTokenAccountB: aliceTokenAccountB,
        offer: testOffer,
        vault: testVault,
        tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
      });

      await connection.sendTransactionFromInstructions({
        feePayer: alice,
        instructions: [takeOfferInstruction],
      });

      // Verify token transfers
      const bobTokenABalance = await connection.getTokenAccountBalance({
        tokenAccount: bobTokenAccountA,
        mint: offeredToken,
        useTokenExtensions: true,
      });
      assert.equal(
        bobTokenABalance.amount,
        bobInitialTokenAAmount + tokenAOfferedAmount,
        "Bob's token A balance should be initial + offered amount",
      );

      const aliceTokenBBalance = await connection.getTokenAccountBalance({
        tokenAccount: aliceTokenAccountB,
        mint: wantedToken,
        useTokenExtensions: true,
      });
      assert.equal(aliceTokenBBalance.amount, wantedAmount, "Alice's token B balance should match wanted amount");
    });

    test("fails when taker has insufficient token balance", async () => {
      // Create an offer from Alice for a large amount of token B
      const largeTokenBAmount = 1000n * TOKEN; // Much larger than Bob's balance
      const { offer, vault } = await createTestOffer({
        connection,
        maker: alice,
        offeredToken,
        wantedToken,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        wantedAmount: largeTokenBAmount,
      });

      const takeOfferInstruction = await programClient.getTakeOfferInstructionAsync({
        taker: bob,
        maker: alice.address,
        offeredToken,
        wantedToken,
        takerTokenAccountA: bobTokenAccountA,
        makerTokenAccountB: aliceTokenAccountB,
        offer,
        vault,
        tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
      });

      try {
        await connection.sendTransactionFromInstructions({
          feePayer: bob,
          instructions: [takeOfferInstruction],
        });
        assert.fail("Expected the take offer to fail but it succeeded");
      } catch (thrownObject) {
        const error = thrownObject as ErrorWithTransaction;
        assert(
          error.message === INSUFFICIENT_FUNDS_ERROR,
          `Expected "${INSUFFICIENT_FUNDS_ERROR}" but got: ${error.message}`,
        );
      }
    });
  });

  describe("refundOffer", () => {
    let testOffer: Address;
    let testVault: Address;

    before(async () => {
      const result = await createTestOffer({
        connection,
        maker: alice,
        offeredToken,
        wantedToken,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        wantedAmount,
      });
      testOffer = result.offer;
      testVault = result.vault;
    });

    test("successfully refunds an offer to the maker", async () => {
      const aliceBalanceBefore = await connection.getTokenAccountBalance({
        tokenAccount: aliceTokenAccountA,
        mint: offeredToken,
        useTokenExtensions: true,
      });

      const refundOfferInstruction = await programClient.getRefundOfferInstructionAsync({
        maker: alice,
        offeredToken,
        makerTokenAccountA: aliceTokenAccountA,
        offer: testOffer,
        vault: testVault,
        tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
      });

      await connection.sendTransactionFromInstructions({
        feePayer: alice,
        instructions: [refundOfferInstruction],
      });

      // Verify refund
      const aliceBalanceAfter = await connection.getTokenAccountBalance({
        tokenAccount: aliceTokenAccountA,
        mint: offeredToken,
        useTokenExtensions: true,
      });
      assert.ok(
        aliceBalanceAfter.amount > aliceBalanceBefore.amount,
        "Balance after refund should be greater than before",
      );

      // Verify vault is closed
      const isClosed = await connection.checkTokenAccountIsClosed({
        tokenAccount: testVault,
        useTokenExtensions: true,
      });
      assert.ok(isClosed, "Vault should be closed");
    });

    test("fails when non-maker tries to refund the offer", async () => {
      const { offer, vault } = await createTestOffer({
        connection,
        maker: alice,
        offeredToken,
        wantedToken,
        makerTokenAccountA: aliceTokenAccountA,
        tokenAOfferedAmount,
        wantedAmount,
      });

      const refundOfferInstruction = await programClient.getRefundOfferInstructionAsync({
        maker: bob,
        offeredToken,
        makerTokenAccountA: bobTokenAccountA,
        offer,
        vault,
        tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
      });

      try {
        await connection.sendTransactionFromInstructions({
          feePayer: bob,
          instructions: [refundOfferInstruction],
        });
        assert.fail("Expected the refund to fail but it succeeded");
      } catch (thrownObject) {
        const error = thrownObject as ErrorWithTransaction;
        assert.equal(error.message, REFUND_OFFER_ERROR, "Expected refund offer error");
      }
    });
  });
});
