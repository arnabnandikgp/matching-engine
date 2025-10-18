import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair } from "@solana/web3.js";
import { MatchingEngine } from "../../target/types/matching_engine";
import * as fs from "fs";
import {
  getCompDefAccOffset,
  getArciumAccountBaseSeed,
  getArciumProgAddress,
  uploadCircuit,
  buildFinalizeCompDefTx,
  getMXEAccAddress,
} from "@arcium-hq/client";

/**
 * Initialize submit_order computation definition
 */
export async function initSubmitOrderCompDef(
  program: Program<MatchingEngine>,
  owner: Keypair,
  uploadRawCircuit: boolean = false,
  offchainSource: boolean = false
): Promise<string> {
  const baseSeedCompDefAcc = getArciumAccountBaseSeed(
    "ComputationDefinitionAccount"
  );
  const offset = getCompDefAccOffset("submit_order");

  const compDefPDA = PublicKey.findProgramAddressSync(
    [baseSeedCompDefAcc, program.programId.toBuffer(), offset],
    getArciumProgAddress()
  )[0];

  console.log("Submit order comp def PDA:", compDefPDA.toBase58());

  const sig = await program.methods
    .initSubmitOrderCompDef()
    .accounts({
      compDefAccount: compDefPDA,
      payer: owner.publicKey,
      mxeAccount: getMXEAccAddress(program.programId),
    })
    .signers([owner])
    .rpc({
      commitment: "confirmed",
    });

  console.log("Init submit_order computation definition tx:", sig);

  const provider = program.provider as anchor.AnchorProvider;

  if (uploadRawCircuit) {
    const rawCircuit = fs.readFileSync("build/submit_order.arcis");
    await uploadCircuit(
      provider,
      "submit_order",
      program.programId,
      rawCircuit,
      true
    );
  } else if (!offchainSource) {
    const finalizeTx = await buildFinalizeCompDefTx(
      provider,
      Buffer.from(offset).readUInt32LE(),
      program.programId
    );

    const latestBlockhash = await provider.connection.getLatestBlockhash();
    finalizeTx.recentBlockhash = latestBlockhash.blockhash;
    finalizeTx.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;

    finalizeTx.sign(owner);

    await provider.sendAndConfirm(finalizeTx);
  }

  return sig;
}

/**
 * Initialize match_orders computation definition
 */
export async function initMatchOrdersCompDef(
  program: Program<MatchingEngine>,
  owner: Keypair,
  uploadRawCircuit: boolean = false,
  offchainSource: boolean = false
): Promise<string> {
  const baseSeedCompDefAcc = getArciumAccountBaseSeed(
    "ComputationDefinitionAccount"
  );
  const offset = getCompDefAccOffset("match_orders");

  const compDefPDA = PublicKey.findProgramAddressSync(
    [baseSeedCompDefAcc, program.programId.toBuffer(), offset],
    getArciumProgAddress()
  )[0];

  console.log("Match orders comp def PDA:", compDefPDA.toBase58());

  const sig = await program.methods
    .initMatchOrdersCompDef()
    .accounts({
      compDefAccount: compDefPDA,
      payer: owner.publicKey,
      mxeAccount: getMXEAccAddress(program.programId),
    })
    .signers([owner])
    .rpc({
      commitment: "confirmed",
    });

  console.log("Init match_orders computation definition tx:", sig);

  const provider = program.provider as anchor.AnchorProvider;

  if (uploadRawCircuit) {
    const rawCircuit = fs.readFileSync("build/match_orders.arcis");
    await uploadCircuit(
      provider,
      "match_orders",
      program.programId,
      rawCircuit,
      true
    );
  } else if (!offchainSource) {
    const finalizeTx = await buildFinalizeCompDefTx(
      provider,
      Buffer.from(offset).readUInt32LE(),
      program.programId
    );

    const latestBlockhash = await provider.connection.getLatestBlockhash();
    finalizeTx.recentBlockhash = latestBlockhash.blockhash;
    finalizeTx.lastValidBlockHeight = latestBlockhash.lastValidBlockHeight;

    finalizeTx.sign(owner);

    await provider.sendAndConfirm(finalizeTx);
  }

  return sig;
}

/**
 * Read keypair from JSON file
 */
export function readKpJson(path: string): Keypair {
  const file = fs.readFileSync(path);
  return Keypair.fromSecretKey(new Uint8Array(JSON.parse(file.toString())));
}
