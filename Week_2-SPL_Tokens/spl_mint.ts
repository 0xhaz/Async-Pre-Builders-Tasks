import web3 from "@solana/web3.js";
import { mintTo, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import fs from "fs";

const wallet = JSON.parse(
  fs.readFileSync("/Users/haz/.config/solana/id.json", "utf8"),
);
const { Keypair, Connection, PublicKey } = web3;
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

const token_decimals = 1_000_000n;

const mint = new PublicKey("8KP3qLzJUEtzhxyapGK7MvwQWTtdPHERDQLVRVjqf6hr");

(async () => {
  try {
    const ata = await getOrCreateAssociatedTokenAccount(
      connection,
      keypair,
      mint,
      keypair.publicKey,
    );
    console.log("Associated Token Account:", ata.address.toBase58());

    //mint to the ata
    const mintTx = await mintTo(
      connection,
      keypair,
      mint,
      ata.address,
      keypair.publicKey,
      Number(100n * token_decimals),
    );
    console.log("Minted 100 tokens. Transaction Signature:", mintTx);
  } catch (err) {
    console.error("Something went wrong:", err);
  }
})();
