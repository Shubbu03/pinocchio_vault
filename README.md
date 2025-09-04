# Pinocchio Vault

A Solana program for creating and managing personal vaults using the Pinocchio framework.

## Features

- **Initialize Vault**: Create a new vault with PDA-based ownership
- **Deposit**: Transfer SOL to your vault
- **Withdraw**: Transfer SOL from your vault (with rent protection)
- **Close**: Close the vault and recover rent

## Program ID

```
63vgRZotq9C4krvqWcVjWHgw1gaZTXuYu76sSbosq6ca
```

## Instructions

| Instruction | Discriminator | Description |
|-------------|---------------|-------------|
| `Init` | 0 | Initialize a new vault |
| `Deposit` | 1 | Deposit SOL to vault |
| `Withdraw` | 2 | Withdraw SOL from vault |
| `Close` | 3 | Close vault and recover rent |

## Project Structure

```
src/
├── lib.rs                 # Main library with no_std support
├── entrypoint.rs          # Program entrypoint and instruction routing
├── errors.rs              # Custom error definitions
├── instructions/          # Program instruction implementations
│   ├── mod.rs            # Instruction module exports
│   ├── init.rs           # Initialize vault instruction
│   ├── deposit.rs        # Deposit SOL instruction
│   ├── withdraw.rs       # Withdraw SOL instruction
│   └── close.rs          # Close vault instruction
└── states/               # Account state definitions
    ├── mod.rs            # State module exports
    ├── state.rs          # VaultState struct and methods
    └── utils.rs          # Utility functions for data loading
```

## States

### VaultState

The core state structure stored in each vault account:

```rust
pub struct VaultState {
    pub owner: Pubkey,  // 32 bytes
}
```

- **Size**: 32 bytes (Pubkey)
- **PDA Seed**: `"vault"`
- **Validation**: Includes PDA validation and owner verification
- **Methods**: `initialize()`, `validate_pda()`

## Building

```bash
chio build
```

## Testing

WIP

## Special Thanks

This project was scaffolded using [Chio CLI](https://github.com/4rjunc/solana-chio) - a blazingly fast tool for setting up Solana Pinocchio projects. Thanks to [@4rjunc](https://github.com/4rjunc) for creating this amazing developer experience! 🚀
