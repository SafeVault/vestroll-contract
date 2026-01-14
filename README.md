# VestRoll Payroll System - Smart Contracts

Payroll smart contracts supporting automated salary distribution, milestone-based payments and financial escrow on the Stellar network using Soroban.

## Technology Stack

### Stellar Ecosystem

- **Language**: Rust (Soroban)
- **Framework**: Soroban SDK 22.0.0
- **Testing**: Rust test framework (`cargo test`)
- **Networks**: Stellar Mainnet, Testnet, Futurenet
- **Tools**: Stellar CLI

## Project Structure

```text
vestroll-contract/
└── vestroll-soroban/
    ├── contracts/
    │   ├── common/          # Shared types, enums, and utility structures
    │   ├── vault/           # Stablecoin escrow and payout logic
    │   ├── lifecycle/       # Contract management (Fixed, Milestone, PAYG)
    │   └── profile/         # Organization and Worker identity/roles
    └── Cargo.toml           # Workspace configuration
```

## Contracts Overview

### 1. Common (`vestroll-common`)

A library crate containing shared data structures used across all contracts. It defines `ContractType`, `ContractStatus`, and `ContractMetadata`.

### 2. Vault (`vestroll-vault`)

Handles the financial core of the system.

- **Escrow**: Securely holds stablecoins (USDC/USDT).
- **Payouts**: Executes transfers to employees/contractors based on authorized triggers.

### 3. Lifecycle (`vestroll-lifecycle`)

Manages the business logic of payroll agreements.

- Supports **Fixed Rate**, **Milestone**, and **Pay-as-you-go** flows.
- Tracks contract state and transitions.

### 4. Profile (`vestroll-profile`)

Manages the decentralized identity of participants.

- **Organizations**: Handles employer entities and their administrative roles.
- **Workers**: Manages employee/contractor profiles and linked wallet addresses.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Stellar CLI](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup#install-the-stellar-cli)

### Setup

1. **Clone the repository**:

   ```bash
   git clone https://github.com/SafeVault/vestroll-contract.git
   cd vestroll-contract/vestroll-soroban
   ```

2. **Build the contracts**:

   ```bash
   stellar contract build
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

## Deployment

To deploy a contract to the Stellar Testnet:

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/vestroll_vault.wasm \
  --source-account <YOUR_ACCOUNT> \
  --network testnet
```
