#!/usr/bin/env bash
set -euo pipefail

PROGRAM_ID="GJmkK7megeRLsTd1XuPGqXZh7ZFusAzKqdUqNx5Gqyb9"
KEYPAIR="${SOLANA_KEYPAIR:-$HOME/.config/solana/id.json}"
RPC_URL="https://api.devnet.solana.com"

echo "=== Capstone Project — Devnet Deployment ==="
echo ""

# 1. Check prerequisites
for cmd in solana anchor; do
  if ! command -v "$cmd" &>/dev/null; then
    echo "ERROR: '$cmd' not found. Please install it first."
    exit 1
  fi
done

# 2. Point Solana CLI to devnet
echo "[1/6] Setting Solana CLI to devnet..."
solana config set --url "$RPC_URL" --keypair "$KEYPAIR" >/dev/null

# 3. Show wallet info and check balance
WALLET=$(solana address)
BALANCE=$(solana balance --lamports | awk '{print $1}')
echo "[2/6] Wallet:  $WALLET"
echo "       Balance: $(solana balance)"

MIN_LAMPORTS=4000000000 # ~4 SOL needed for program deploy
if [ "$BALANCE" -lt "$MIN_LAMPORTS" ]; then
  echo ""
  echo "       Insufficient balance. Requesting airdrop..."
  solana airdrop 5 --url "$RPC_URL" || {
    echo "WARNING: Airdrop failed (rate-limited). Fund your wallet manually:"
    echo "         https://faucet.solana.com"
    echo "         Wallet: $WALLET"
    exit 1
  }
  echo "       New balance: $(solana balance)"
fi

# 4. Build the program
echo "[3/6] Building program..."
anchor build

# 5. Verify program ID matches
BUILT_ID=$(solana-keygen pubkey target/deploy/capstone_project-keypair.json 2>/dev/null || echo "")
if [ -n "$BUILT_ID" ] && [ "$BUILT_ID" != "$PROGRAM_ID" ]; then
  echo "WARNING: Built keypair ($BUILT_ID) differs from declared ID ($PROGRAM_ID)."
  echo "         Run: anchor keys sync"
  exit 1
fi

# 6. Deploy to devnet
echo "[4/6] Deploying to devnet..."
anchor deploy --provider.cluster devnet --provider.wallet "$KEYPAIR"

# 7. Verify deployment
echo "[5/6] Verifying deployment..."
if solana program show "$PROGRAM_ID" --url "$RPC_URL" &>/dev/null; then
  echo "       Program deployed successfully!"
else
  echo "ERROR: Program not found on-chain after deploy."
  exit 1
fi

# 8. Print summary
echo "[6/6] Done!"
echo ""
echo "  Program ID: $PROGRAM_ID"
echo "  Cluster:    devnet"
echo "  Explorer:   https://explorer.solana.com/address/${PROGRAM_ID}?cluster=devnet"
echo ""
echo "To run tests against devnet:"
echo "  anchor test --provider.cluster devnet --skip-local-validator"
