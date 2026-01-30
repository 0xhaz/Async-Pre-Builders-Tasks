import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { createMetadataAccountV3 } from "@metaplex-foundation/mpl-token-metadata";
import {
  createSignerFromKeypair,
  signerIdentity,
  publicKey,
} from "@metaplex-foundation/umi";

import fs from "fs";

const wallet = JSON.parse(
  fs.readFileSync("/Users/haz/.config/solana/id.json", "utf8"),
);

(async () => {
  try {
    const umi = createUmi("https://api.devnet.solana.com");

    const keypair = umi.eddsa.createKeypairFromSecretKey(
      new Uint8Array(wallet),
    );
    const signer = createSignerFromKeypair(umi, keypair);
    umi.use(signerIdentity(signer));

    const mintAddress = publicKey(
      "8KP3qLzJUEtzhxyapGK7MvwQWTtdPHERDQLVRVjqf6hr",
    );

    await createMetadataAccountV3(umi, {
      mint: mintAddress,
      mintAuthority: signer,
      payer: signer,
      updateAuthority: keypair.publicKey,
      data: {
        name: "Haz Turbin3 Token",
        symbol: "HTT",
        uri: "",
        sellerFeeBasisPoints: 0,
        creators: null,
        collection: null,
        uses: null,
      },
      isMutable: true,
      collectionDetails: null,
    }).sendAndConfirm(umi);

    console.log("Metadata created successfully!");
    console.log("Name: Haz Turbin3 Token");
    console.log("Symbol: HTT");
  } catch (err) {
    console.error("Error:", err);
  }
})();
