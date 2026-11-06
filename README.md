# turbine_vault

1. Creates a per-user SOL vault (`initialize`)
2. Accepts deposits into the vault (`deposit`)
3. Pays out withdrawals signed by the vault PDA (`withdraw`)
4. Drains the vault and closes the state account (`close`)

Each user gets two PDAs: `vault_state` (stores PDA bumps) and `vault` (holds raw SOL).

## Setup

Requirements: Rust 1.89+, Solana CLI, Anchor CLI, a funded localnet wallet.

1. Wallet for the provider (see `Anchor.toml`):

```
~/.config/solana/id.json
```

1. Build the program (also refreshes `target/deploy/turbine_vault.so` used by the tests):

```bash
anchor build
```

## Commands

```bash
anchor build                          # build the program
cargo test -p turbine_vault           # run all LiteSVM tests
```

## Run

Tests run against LiteSVM with the freshly built `.so` — rebuild after any `src/` change or the tests exercise stale code.

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

Moves `amount` in lamports from vault to user, signed by the vault PDA (`seeds + vault_bump` via `new_with_signer`).
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
