## Solana Introductions

## Accounts

- All on-chain data lives in Accounts
- Dynamic sized, can grow as required
- Can only be created by the System Program
- Account Flags
  - Writable - serial access, (one at a time)
  - Read only - parallel access, (multiple simultaneously)
  - Signer - a signer of the transaction
  - Executable - program account

Example:

```json
{
    key: PublicKey,
    lamports: number,
    data: Uint8Array,
    is_executable: boolean,
    owner: PublicKey
}
```

## Programs

- Executable: Marked as executable account
- Stateless: Hold only compiled code
- Owned by Loaders: Default is Upgradeable Loader
- Ownership: Can own non-executable accounts
- Access: Via program_id
- Native Programs: Built-in (or BPF) by Solana
- User Programs: Custom-developed by builders - us

## Rent

- Rent must be paid to create accounts
- Pay 2 years up-front for Rent-Exemption
- Rent-exemption is required on Account creation
- Closing an Account allows rent to be reclaimed
- Resizing an Account costs / returns the difference
- Upgradable Programs require 4 years of rent upfront

## Transactions

- Accounts: Must reference all accounts involved
- Composition: One or more instructions
- Instructions interface with Solana programs
- Atomicity: Fails entirely if any instruction fails

```json
{
    message: {
        instructions: Array<Instructions>,
        recent_blockhash: number,
        fee_payer: PublicKey
    }
    signers: Array<Uint8Array>
}
```

## Compute

- Consumption: All on-chain actions use compute units
- Block Limit: Fixed maximum per block
- Extra Requests: Possible if needed, up to 1.4mil
  - Avoid unless necessary
  - Aim to request only what's needed
  - No automatic higher fee required
  - Priority fees possible

## PDA

- Made up of seeds and a bump
- Deterministic if seeds are fixed (e.g Associated Token Accounts)
- Can't collide with PDAs or Account created by other programs
- Can be used as hashmap (key / value)
- PDA account pubkeys resembles accounts but no private key
- Can authorize / sign on program's behalf

## IDL

- Interface Design Language
- Many on-chain programs have an IDL
- Makes interacting with on-chain programs much easier
- Public IDLs can be uploded to the chain for easy etc.
- Written in JSON
