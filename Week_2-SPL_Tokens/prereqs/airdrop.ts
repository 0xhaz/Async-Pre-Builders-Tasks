import web3, { LAMPORTS_PER_SOL } from "@solana/web3.js";
import fs from "fs";

const wallet = JSON.parse(
  fs.readFileSync("/Users/haz/.config/solana/id.json", "utf8"),
);

const { Keypair, Connection, PublicKey } = web3;
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));
const commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

(async () => {
  try {
    const txhash = await connection.requestAirdrop(
      keypair.publicKey,
      2 * LAMPORTS_PER_SOL,
    );

    console.log(`Success! Check out your TX here:
        https://explorer.solana.com/tx/${txhash}?cluster=devnet
        `);
  } catch (e) {
    console.error(`Airdrop failed: ${e}`);
  }
})();
