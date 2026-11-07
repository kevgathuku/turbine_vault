# turbine_vault

[![CI](https://github.com/kevgathuku/turbine_vault/actions/workflows/ci.yml/badge.svg)](https://github.com/kevgathuku/turbine_vault/actions/workflows/ci.yml)

1. Creates a per-user SOL vault (`initialize`)
2. Accepts deposits into the vault (`deposit`)
3. Pays out withdrawals signed by the vault PDA (`withdraw`)
4. Drains the vault and closes the state account (`close`)

Each user gets two PDAs: `vault_state` (stores PDA bumps) and `vault` (holds raw SOL).

## Setup

Requirements: Rust 1.91, Solana CLI, Anchor CLI, a funded localnet wallet.

Wallet for the provider (see `Anchor.toml`):

```txt
~/.config/solana/id.json
```

## Commands

```bash
# build the program
anchor build          
# run all LiteSVM tests      
cargo test -p turbine_vault           
```

## Run

Tests run against the freshly built `.so` — rebuild after any `src/` change

1. Initialize a vault for your wallet:

```rust
initialize_tx(&user.pubkey(), &vault, &vault_state)
```

Creates the `vault_state` PDA (`seeds = ["state", user]`, stores both bumps) and funds the `vault` PDA (`seeds = ["vault", user]`) with exactly the rent-exempt minimum via system transfer.

1. Deposit SOL:

```rust
deposit_tx(&user.pubkey(), &vault, &vault_state, amount)
```

Moves `amount` in lamports from user to vault
Fails on invalid amounts:

- `amount == 0` => `InvalidAmount`
- `amount` > user balance  => `InsufficientFunds`

1. Withdraw SOL:

```rust
withdraw_tx(&user.pubkey(), &vault, &vault_state, amount)
```

Moves `amount` in lamports from vault to user, signed by the vault PDA (`seeds + vault_bump`).
Fails on invalid amounts:

- `amount == 0` => `InvalidAmount`
- `amount` > vault balance  => `InsufficientVaultFunds`

1. Close the vault:

```rust
close_tx(&user.pubkey(), &vault, &vault_state)
```

Drains the full vault balance back to the user, then closes `vault_state`, refunding its rent

## Notes

- Tests build instructions through shared helpers in `programs/turbine_vault/tests/common/mod.rs`
