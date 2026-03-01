import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { CapstoneProject } from "../target/types/capstone_project";
import { PublicKey } from "@solana/web3.js";

module.exports = async function (provider: anchor.AnchorProvider) {
  anchor.setProvider(provider);

  const program = anchor.workspace
    .capstoneProject as Program<CapstoneProject>;
  const wallet = provider.wallet as anchor.Wallet;

  console.log("Program ID:", program.programId.toBase58());
  console.log("Wallet:    ", wallet.publicKey.toBase58());

  const [vaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), wallet.publicKey.toBuffer()],
    program.programId
  );

  console.log("Vault PDA: ", vaultPda.toBase58());

  // Check if vault already exists
  const vaultAccount = await provider.connection.getAccountInfo(vaultPda);
  if (vaultAccount) {
    console.log("Vault already initialized. Skipping.");
    return;
  }

  // Initialize vault
  const emergencyAdmin = wallet.publicKey; // use deployer as emergency admin
  console.log("Initializing vault...");

  await program.methods
    .initialize()
    .accountsPartial({
      authority: wallet.publicKey,
      emergencyAdmin: emergencyAdmin,
    })
    .rpc();

  const vault = await program.account.vault.fetch(vaultPda);
  console.log("Vault initialized!");
  console.log("  Authority:       ", vault.authority.toBase58());
  console.log("  Emergency Admin: ", vault.emergencyAdmin.toBase58());
  console.log("  Bump:            ", vault.bump);
};
