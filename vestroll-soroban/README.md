# VestRoll Payroll System - Smart Contracts

Payroll smart contracts supporting automated salary distribution, milestone-based payments and financial escrow on the Stellar network using Soroban.
VestRoll is a Payroll management platform built for modern businesses. it provides a smooth experience for managing contracts, team members, and financial operations. By leveraging the Stellar configuration, VestRoll ensures fast, low-cost, and secure stablecoin interactions, making global payroll efficient and accessible.

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
├── contracts/
│   ├── common/          # Shared types, enums, and utility structures
│   ├── vault/           # Stablecoin escrow and payout logic
│   ├── lifecycle/       # Contract management (Fixed, Milestone, PAYG)
│   └── identity/        # Organization and Worker identity/roles
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

### 4. Identity (`vestroll-identity`)

Manages the decentralized identity of participants.

- **Organizations**: Handles employer entities and their administrative roles.
- **Workers**: Manages employee/contractor profiles and linked wallet addresses.

## 🎯 Target Audience & Ecosystem Impact

### Who is this for?

- **Global Enterprises**: Companies with distributed teams needing seamless cross-border payroll.
- **DAO & Web3 Organizations**: Native crypto organizations requiring fiat and stablecoin payroll solutions.
- **Freelancers & Contractors**: Individuals seeking transparent, instant, and low-fee payments.

### Contribution to the Stellar Ecosystem

VestRoll plays a pivotal role in the **Stellar ecosystem** by:

1.  **Driving Real-World Utility**: Moving beyond speculation to practical, high-volume stablecoin use cases (Payroll).
2.  **Highlighting Efficiency**: Showcasing Stellar's speed and low fees for frequent, small-to-large value transactions.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Stellar CLI](https://developers.stellar.org/docs/build/smart-contracts/getting-started/setup#install-the-stellar-cli)

### Setup

1. **Clone the repository**:

   ```bash
   git clone https://github.com/SafeVault/vestroll-contract.git
   cd vestroll-contract-emrys
   ```

2. **Build the contracts**:

   ```bash
   make build
   ```

3. **Run tests**:
   ```bash
   make test
   ```
