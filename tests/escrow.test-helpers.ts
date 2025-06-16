import { Connection } from "solana-kite";
import { lamports, type KeyPairSigner, type Address } from "@solana/kit";
import * as programClient from "../dist/js-client";
import { TOKEN_EXTENSIONS_PROGRAM } from "solana-kite";

export const log = console.log;
export const stringify = (object: any) => {
  const bigIntReplacer = (key: string, value: any) => (typeof value === "bigint" ? value.toString() : value);
  return JSON.stringify(object, bigIntReplacer, 2);
};

export const ONE_SOL = lamports(1n * 1_000_000_000n);

export const getRandomBigInt = () => {
  return BigInt(Math.floor(Math.random() * 1_000_000_000_000_000_000));
};

// Helper function to create a test offer
export async function createTestOffer(params: {
  connection: Connection;
  maker: KeyPairSigner;
  offeredToken: Address;
  wantedToken: Address;
  makerTokenAccountA: Address;
  tokenAOfferedAmount: bigint;
  wantedAmount: bigint;
  offerId?: bigint;
}) {
  const {
    connection,
    maker,
    offeredToken,
    wantedToken,
    makerTokenAccountA,
    tokenAOfferedAmount,
    wantedAmount,
    offerId = getRandomBigInt(),
  } = params;

  const offerPDAAndBump = await connection.getPDAAndBump(programClient.ESCROW_PROGRAM_ADDRESS, ["offer", offerId]);
  const offer = offerPDAAndBump.pda;
  const vault = await connection.getTokenAccountAddress(offer, offeredToken, true);

  const makeOfferInstruction = await programClient.getMakeOfferInstructionAsync({
    maker,
    offeredToken,
    wantedToken,
    makerTokenAccountA,
    offer,
    vault,
    id: offerId,
    tokenAOfferedAmount,
    wantedAmount,
    tokenProgram: TOKEN_EXTENSIONS_PROGRAM,
  });

  const signature = await connection.sendTransactionFromInstructions({
    feePayer: maker,
    instructions: [makeOfferInstruction],
  });

  return { offer, vault, offerId, signature };
}
