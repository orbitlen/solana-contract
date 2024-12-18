import * as path from "path";
import { promises as fs } from "fs";

import {
  PublicKey,
  LAMPORTS_PER_SOL,
  Connection,
  Signer,
} from "@solana/web3.js";

export function delay(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Airdrop SOL to an address.
 */
export async function safeAirdrop(address: PublicKey, connection: Connection) {
  const accountInfo = await connection.getAccountInfo(address, "confirmed");
  let balance = await connection.getBalance(address);

  if (accountInfo != null || balance < LAMPORTS_PER_SOL) {
    console.log(`User: ${address} have SOL ${balance / LAMPORTS_PER_SOL}`);
    let signature = await connection.requestAirdrop(
      address,
      LAMPORTS_PER_SOL * 10
    );
    await connection.confirmTransaction(signature);
    let newBalance = await connection.getBalance(address);
    console.log(`Airdropped ${(newBalance - balance) / LAMPORTS_PER_SOL} SOL`);
  }
}
