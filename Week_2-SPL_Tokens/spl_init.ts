import web3 from "@solana/web3.js";
import { createMint } from "@solana/spl-token";
import fs from "fs";
const wallet = JSON.parse(
  fs.readFileSync("/Users/haz/.config/solana/id.json", "utf8"),
);

const { Keypair, Connection } = web3;
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));
const commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

(async () => {
  try {
    const mint = await createMint(
      connection,
      keypair,
      keypair.publicKey,
      null,
      6,
    );
    console.log("mint sucessfully created");
  } catch (err) {
    console.error("something went wrong", err);
  }
})();
