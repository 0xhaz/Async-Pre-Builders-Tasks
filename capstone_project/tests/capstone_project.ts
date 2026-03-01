import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { CapstoneProject } from "../target/types/capstone_project";
import {
  createMint,
  createAssociatedTokenAccount,
  createAssociatedTokenAccountInstruction,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddressSync,
} from "@solana/spl-token";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { expect } from "chai";

describe("capstone_project", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .capstoneProject as Program<CapstoneProject>;
  const wallet = provider.wallet as anchor.Wallet;

  const emergencyAdmin = Keypair.generate();
  const owner2 = Keypair.generate();
  const owner3 = Keypair.generate();

  let vaultPda: PublicKey;
  let vaultBump: number;
  let tokenMint: PublicKey;
  let userAta: PublicKey;
  let vaultAta: PublicKey;

  before(async () => {
    [vaultPda, vaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), wallet.publicKey.toBuffer()],
      program.programId
    );

    // Airdrop to additional signers
    for (const kp of [owner2, owner3]) {
      const sig = await provider.connection.requestAirdrop(
        kp.publicKey,
        2 * LAMPORTS_PER_SOL
      );
      const latestBlockhash =
        await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction({
        signature: sig,
        ...latestBlockhash,
      });
    }
  });

  // =========================================================
  // Happy-path tests
  // =========================================================

  it("initializes vault", async () => {
    await program.methods
      .initialize()
      .accountsPartial({
        authority: wallet.publicKey,
        emergencyAdmin: emergencyAdmin.publicKey,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.authority.toBase58()).to.equal(wallet.publicKey.toBase58());
    expect(vault.emergencyAdmin.toBase58()).to.equal(
      emergencyAdmin.publicKey.toBase58()
    );
    expect(vault.paused).to.equal(false);
    expect(vault.bump).to.equal(vaultBump);
  });

  it("adds a supported token", async () => {
    tokenMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6
    );

    vaultAta = getAssociatedTokenAddressSync(tokenMint, vaultPda, true);

    await program.methods
      .addSupportedToken()
      .accountsPartial({
        vault: vaultPda,
        vaultTokenAccount: vaultAta,
        tokenMint: tokenMint,
        authority: wallet.publicKey,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.supportedTokens.length).to.equal(1);
    expect(vault.supportedTokens[0].mint.toBase58()).to.equal(
      tokenMint.toBase58()
    );
    expect(vault.supportedTokens[0].isActive).to.equal(true);
  });

  it("deposits tokens", async () => {
    userAta = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      tokenMint,
      wallet.publicKey
    );

    await mintTo(
      provider.connection,
      wallet.payer,
      tokenMint,
      userAta,
      wallet.publicKey,
      1_000_000_000
    );

    const depositAmount = new BN(500_000_000);

    await program.methods
      .deposit(depositAmount)
      .accountsPartial({
        vault: vaultPda,
        userTokenAccount: userAta,
        vaultTokenAccount: vaultAta,
        userAuthority: wallet.publicKey,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.totalValueLocked.toNumber()).to.equal(500_000_000);
    expect(vault.tokenBalances.length).to.equal(1);
    expect(vault.tokenBalances[0].balance.toNumber()).to.equal(500_000_000);

    const vaultTokenAcct = await getAccount(provider.connection, vaultAta);
    expect(Number(vaultTokenAcct.amount)).to.equal(500_000_000);
  });

  it("withdraws tokens", async () => {
    const withdrawAmount = new BN(200_000_000);

    await program.methods
      .withdraw(withdrawAmount)
      .accountsPartial({
        vault: vaultPda,
        vaultTokenAccount: vaultAta,
        userTokenAccount: userAta,
        userAuthority: wallet.publicKey,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.totalValueLocked.toNumber()).to.equal(300_000_000);
    expect(vault.tokenBalances[0].balance.toNumber()).to.equal(300_000_000);

    const userTokenAcct = await getAccount(provider.connection, userAta);
    expect(Number(userTokenAcct.amount)).to.equal(700_000_000);
  });

  it("withdraws SOL from vault", async () => {
    // Fund vault with SOL
    const fundTx = new anchor.web3.Transaction().add(
      SystemProgram.transfer({
        fromPubkey: wallet.publicKey,
        toPubkey: vaultPda,
        lamports: LAMPORTS_PER_SOL,
      })
    );
    await provider.sendAndConfirm(fundTx);

    const withdrawAmount = new BN(100_000);

    await program.methods
      .withdrawSol(withdrawAmount)
      .accountsPartial({
        vault: vaultPda,
        recipient: wallet.publicKey,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.totalFeesCollected.toNumber()).to.equal(0); // 0 fees (fee_bps = 0)
  });

  it("transfers SOL (authority-gated)", async () => {
    const recipientBalBefore = await provider.connection.getBalance(
      owner2.publicKey
    );

    const transferAmount = new BN(50_000);

    await program.methods
      .transferSol(transferAmount)
      .accountsPartial({
        vault: vaultPda,
        recipient: owner2.publicKey,
        authority: wallet.publicKey,
      })
      .rpc();

    const recipientBalAfter = await provider.connection.getBalance(
      owner2.publicKey
    );
    expect(recipientBalAfter - recipientBalBefore).to.equal(50_000);
  });

  it("initializes multisig", async () => {
    const owners = [wallet.publicKey, owner2.publicKey, owner3.publicKey];
    const threshold = new BN(2);
    const nonce = 1;

    await program.methods
      .initializeMultiSig(owners, threshold, nonce)
      .accountsPartial({
        vault: vaultPda,
        authority: wallet.publicKey,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.multiSig).to.not.be.null;
    expect(vault.multiSig.owners.length).to.equal(3);
    expect(vault.multiSig.threshold.toNumber()).to.equal(2);
    expect(vault.multiSig.nonce).to.equal(1);
  });

  it("creates a multisig transaction", async () => {
    // Use multisig_signer PDA as `from` for System Transfer
    // (vault PDA carries data and can't be used with system transfer)
    const [multisigSigner] = PublicKey.findProgramAddressSync(
      [vaultPda.toBuffer(), Buffer.from([1])],
      program.programId
    );

    // Fund the multisig signer PDA (needs rent-exempt minimum ~890k lamports)
    const fundTx = new anchor.web3.Transaction().add(
      SystemProgram.transfer({
        fromPubkey: wallet.publicKey,
        toPubkey: multisigSigner,
        lamports: LAMPORTS_PER_SOL,
      })
    );
    await provider.sendAndConfirm(fundTx);

    const txAccounts = [
      { pubkey: multisigSigner, isSigner: false, isWritable: true },
      { pubkey: owner2.publicKey, isSigner: false, isWritable: true },
    ];

    // System program Transfer instruction data: type(u32 LE) + lamports(u64 LE)
    const ixData = Buffer.alloc(12);
    ixData.writeUInt32LE(2, 0);
    ixData.writeBigUInt64LE(BigInt(1000), 4);

    await program.methods
      .createMultisigTransaction(
        SystemProgram.programId,
        txAccounts,
        Buffer.from(ixData)
      )
      .accountsPartial({
        vault: vaultPda,
        proposer: wallet.publicKey,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.multiSigTransactions.length).to.equal(1);
    expect(vault.multiSigTransactions[0].didExecute).to.equal(false);
    expect(vault.multiSigTransactions[0].proposer.toBase58()).to.equal(
      wallet.publicKey.toBase58()
    );
    expect(vault.multiSigTransactions[0].signers[0]).to.equal(true);
    expect(vault.multiSigTransactions[0].signers[1]).to.equal(false);
  });

  it("approves a multisig transaction", async () => {
    await program.methods
      .approveMultisigTransaction(new BN(0))
      .accountsPartial({
        vault: vaultPda,
        approver: owner2.publicKey,
      })
      .signers([owner2])
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.multiSigTransactions[0].signers[0]).to.equal(true);
    expect(vault.multiSigTransactions[0].signers[1]).to.equal(true);
  });

  it("executes a multisig transaction", async () => {
    const [multisigSigner] = PublicKey.findProgramAddressSync(
      [vaultPda.toBuffer(), Buffer.from([1])],
      program.programId
    );

    const owner2BalBefore = await provider.connection.getBalance(owner2.publicKey);

    await program.methods
      .executeMultisigTransaction(new BN(0))
      .accountsPartial({
        vault: vaultPda,
        multisigSigner: multisigSigner,
        executor: wallet.publicKey,
      })
      .remainingAccounts([
        { pubkey: multisigSigner, isSigner: false, isWritable: true },
        { pubkey: owner2.publicKey, isSigner: false, isWritable: true },
        {
          pubkey: SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
      ])
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.multiSigTransactions[0].didExecute).to.equal(true);

    const owner2BalAfter = await provider.connection.getBalance(owner2.publicKey);
    expect(owner2BalAfter - owner2BalBefore).to.equal(1000);
  });

  it("sets multisig owners", async () => {
    const newOwners = [wallet.publicKey, owner2.publicKey];

    await program.methods
      .setMultisigOwners(newOwners)
      .accountsPartial({
        vault: vaultPda,
        authority: wallet.publicKey,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.multiSig.owners.length).to.equal(2);
    expect(vault.multiSig.threshold.toNumber()).to.equal(2);
  });

  it("changes multisig threshold", async () => {
    await program.methods
      .changeMultisigThreshold(new BN(1))
      .accountsPartial({
        vault: vaultPda,
        authority: wallet.publicKey,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    expect(vault.multiSig.threshold.toNumber()).to.equal(1);
  });

  // =========================================================
  // Error tests
  // =========================================================

  it("fails to deposit with unsupported token", async () => {
    const badMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      6
    );

    const badUserAta = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      badMint,
      wallet.publicKey
    );

    await mintTo(
      provider.connection,
      wallet.payer,
      badMint,
      badUserAta,
      wallet.publicKey,
      1_000_000
    );

    const badVaultAta = getAssociatedTokenAddressSync(
      badMint,
      vaultPda,
      true
    );

    try {
      await program.methods
        .deposit(new BN(100_000))
        .accountsPartial({
          vault: vaultPda,
          userTokenAccount: badUserAta,
          vaultTokenAccount: badVaultAta,
          userAuthority: wallet.publicKey,
        })
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err).to.exist;
    }
  });

  it("fails to withdraw more than vault balance", async () => {
    try {
      await program.methods
        .withdraw(new BN(999_999_999_999))
        .accountsPartial({
          vault: vaultPda,
          vaultTokenAccount: vaultAta,
          userTokenAccount: userAta,
          userAuthority: wallet.publicKey,
        })
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.toString()).to.include("InvalidAmount");
    }
  });

  it("fails to withdraw unsupported token", async () => {
    const badMint = await createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      9
    );

    // Create vault ATA for unsupported token (PDA owner is off-curve, must build tx manually)
    const badVaultAta = getAssociatedTokenAddressSync(badMint, vaultPda, true);
    const createAtaIx = createAssociatedTokenAccountInstruction(
      wallet.publicKey,
      badVaultAta,
      vaultPda,
      badMint,
    );
    const ataTx = new anchor.web3.Transaction().add(createAtaIx);
    await provider.sendAndConfirm(ataTx);

    const badUserAta = await createAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      badMint,
      wallet.publicKey
    );

    try {
      await program.methods
        .withdraw(new BN(100))
        .accountsPartial({
          vault: vaultPda,
          vaultTokenAccount: badVaultAta,
          userTokenAccount: badUserAta,
          userAuthority: wallet.publicKey,
        })
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.toString()).to.include("TokenNotSupported");
    }
  });

  it("fails transfer without authority", async () => {
    const fakeAuthority = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      fakeAuthority.publicKey,
      LAMPORTS_PER_SOL
    );
    const latestBlockhash =
      await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: sig,
      ...latestBlockhash,
    });

    try {
      await program.methods
        .transferSol(new BN(1000))
        .accountsPartial({
          vault: vaultPda,
          recipient: owner3.publicKey,
          authority: fakeAuthority.publicKey,
        })
        .signers([fakeAuthority])
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      // has_one = authority constraint fails
      expect(err).to.exist;
    }
  });

  it("fails initialize multisig with threshold 0", async () => {
    const freshAuth = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      freshAuth.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    const bh = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: sig,
      ...bh,
    });

    const [freshVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), freshAuth.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initialize()
      .accountsPartial({
        authority: freshAuth.publicKey,
        emergencyAdmin: emergencyAdmin.publicKey,
      })
      .signers([freshAuth])
      .rpc();

    try {
      await program.methods
        .initializeMultiSig(
          [freshAuth.publicKey, owner2.publicKey],
          new BN(0),
          1
        )
        .accountsPartial({
          vault: freshVault,
          authority: freshAuth.publicKey,
        })
        .signers([freshAuth])
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.toString()).to.include("InvalidThreshold");
    }
  });

  it("fails initialize multisig with threshold > owners", async () => {
    const freshAuth = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      freshAuth.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    const bh = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: sig,
      ...bh,
    });

    const [freshVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), freshAuth.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initialize()
      .accountsPartial({
        authority: freshAuth.publicKey,
        emergencyAdmin: emergencyAdmin.publicKey,
      })
      .signers([freshAuth])
      .rpc();

    try {
      await program.methods
        .initializeMultiSig(
          [freshAuth.publicKey, owner2.publicKey],
          new BN(5),
          1
        )
        .accountsPartial({
          vault: freshVault,
          authority: freshAuth.publicKey,
        })
        .signers([freshAuth])
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.toString()).to.include("InvalidThreshold");
    }
  });

  it("fails initialize multisig with duplicate owners", async () => {
    const freshAuth = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      freshAuth.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    const bh = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: sig,
      ...bh,
    });

    const [freshVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), freshAuth.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initialize()
      .accountsPartial({
        authority: freshAuth.publicKey,
        emergencyAdmin: emergencyAdmin.publicKey,
      })
      .signers([freshAuth])
      .rpc();

    try {
      await program.methods
        .initializeMultiSig(
          [freshAuth.publicKey, freshAuth.publicKey],
          new BN(1),
          1
        )
        .accountsPartial({
          vault: freshVault,
          authority: freshAuth.publicKey,
        })
        .signers([freshAuth])
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.toString()).to.include("DuplicateOwners");
    }
  });

  it("fails to approve already-approved transaction", async () => {
    // Create a fresh transaction so we can test double-approval
    const [multisigSigner] = PublicKey.findProgramAddressSync(
      [vaultPda.toBuffer(), Buffer.from([1])],
      program.programId
    );
    const txAccounts = [
      { pubkey: multisigSigner, isSigner: false, isWritable: true },
      { pubkey: owner3.publicKey, isSigner: false, isWritable: true },
    ];
    const ixData = Buffer.alloc(12);
    ixData.writeUInt32LE(2, 0);
    ixData.writeBigUInt64LE(BigInt(500), 4);

    // wallet auto-approves as proposer (owners are [wallet, owner2, owner3] after restore)
    await program.methods
      .createMultisigTransaction(
        SystemProgram.programId,
        txAccounts,
        Buffer.from(ixData)
      )
      .accountsPartial({
        vault: vaultPda,
        proposer: wallet.publicKey,
      })
      .rpc();

    let vault = await program.account.vault.fetch(vaultPda);
    const txId = vault.multiSigTransactions.length - 1;

    // wallet already approved as proposer; try to approve again
    try {
      await program.methods
        .approveMultisigTransaction(new BN(txId))
        .accountsPartial({
          vault: vaultPda,
          approver: wallet.publicKey,
        })
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.toString()).to.include("TransactionAlreadySigned");
    }
  });

  it("fails to execute with insufficient signatures", async () => {
    // Restore 3 owners with threshold 2
    await program.methods
      .setMultisigOwners([
        wallet.publicKey,
        owner2.publicKey,
        owner3.publicKey,
      ])
      .accountsPartial({
        vault: vaultPda,
        authority: wallet.publicKey,
      })
      .rpc();

    await program.methods
      .changeMultisigThreshold(new BN(2))
      .accountsPartial({
        vault: vaultPda,
        authority: wallet.publicKey,
      })
      .rpc();

    const [multisigSigner] = PublicKey.findProgramAddressSync(
      [vaultPda.toBuffer(), Buffer.from([1])],
      program.programId
    );

    // Create a new tx (wallet auto-approves = 1 of 2)
    const txAccounts = [
      { pubkey: multisigSigner, isSigner: false, isWritable: true },
      { pubkey: owner3.publicKey, isSigner: false, isWritable: true },
    ];
    const ixData = Buffer.alloc(12);
    ixData.writeUInt32LE(2, 0);
    ixData.writeBigUInt64LE(BigInt(1000), 4);

    await program.methods
      .createMultisigTransaction(
        SystemProgram.programId,
        txAccounts,
        Buffer.from(ixData)
      )
      .accountsPartial({
        vault: vaultPda,
        proposer: wallet.publicKey,
      })
      .rpc();

    const vault = await program.account.vault.fetch(vaultPda);
    const txId = vault.multiSigTransactions.length - 1;

    try {
      await program.methods
        .executeMultisigTransaction(new BN(txId))
        .accountsPartial({
          vault: vaultPda,
          multisigSigner: multisigSigner,
          executor: wallet.publicKey,
        })
        .remainingAccounts([
          { pubkey: multisigSigner, isSigner: false, isWritable: true },
          {
            pubkey: owner3.publicKey,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
        ])
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.toString()).to.include("NotEnoughSigners");
    }
  });

  it("fails to change threshold on non-initialized multisig", async () => {
    const freshAuth = Keypair.generate();
    const sig = await provider.connection.requestAirdrop(
      freshAuth.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    const bh = await provider.connection.getLatestBlockhash();
    await provider.connection.confirmTransaction({
      signature: sig,
      ...bh,
    });

    const [freshVault] = PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), freshAuth.publicKey.toBuffer()],
      program.programId
    );

    await program.methods
      .initialize()
      .accountsPartial({
        authority: freshAuth.publicKey,
        emergencyAdmin: emergencyAdmin.publicKey,
      })
      .signers([freshAuth])
      .rpc();

    try {
      await program.methods
        .changeMultisigThreshold(new BN(1))
        .accountsPartial({
          vault: freshVault,
          authority: freshAuth.publicKey,
        })
        .signers([freshAuth])
        .rpc();
      expect.fail("Should have failed");
    } catch (err) {
      expect(err.toString()).to.include("MultisigNotInitialized");
    }
  });
});
