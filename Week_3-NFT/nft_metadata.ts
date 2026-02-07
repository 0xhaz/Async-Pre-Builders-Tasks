import web3 from "@solana/web3.js";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  createGenericFile,
  createSignerFromKeypair,
  signerIdentity,
} from "@metaplex-foundation/umi";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";
import fs from "fs";
const wallet = JSON.parse(
  fs.readFileSync("/Users/haz/.config/solana/id.json", "utf8"),
);

// Create a devnet connection
const umi = createUmi("https://api.devnet.solana.com");

let keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(irysUploader());
umi.use(signerIdentity(signer));

(async () => {
  try {
    // Follow this JSON structure
    // https://docs.metaplex.com/programs/token-metadata/changelog/v1.0#json-structure
    const image =
      "https://gateway.irys.xyz/E816K2wYxQQejdg6z2VYdhsYLQTRXchCMsiswe2rvfHh";
    const metadata = {
      name: "MMMMMMMMMMMMMMMMMMMM",
      symbol: "?",
      description: "?",
      image: image,
      attributes: [{ trait_type: "?", value: "?" }],
      properties: {
        files: [
          {
            type: "image/png",
            uri: "?",
          },
        ],
      },
      creators: [],
    };

    const metadataFile = createGenericFile(
      Buffer.from(JSON.stringify(metadata)),
      "metadata.json",
      { contentType: "application/json" },
    );

    const myUri = await umi.uploader.upload([metadataFile]);
    //  'https://gateway.irys.xyz/7FdYTKCSJRFAy3g42veyAPaUjMVhQHzDn9gjxrmnZoCJ'

    console.log("Your metadata URI: ", myUri);
  } catch (error) {
    console.log("Oops.. Something went wrong", error);
  }
})();
