# VestRoll Payroll System - Smart Contracts

Payroll smart contracts supporting automated salary distribution, milestone-based payments and multi-signature escrow on Ethereum and Stellar networks using stablecoins (USDT, USDC, BUSD).

## Technology Stack

### Ethereum Ecosystem
- **Language**: Solidity ^0.8.20
- **Framework**: Hardhat 2.19+
- **Testing**: Hardhat + Chai + Waffle
- **Libraries**: OpenZeppelin Contracts 5.0+
- **Networks**: Ethereum Mainnet, Goerli, Sepolia, Polygon, BSC
- **Tools**: ethers.js v6, Hardhat-deploy, Slither

### Stellar Ecosystem
- **Language**: Rust (Soroban)
- **Framework**: Soroban SDK 20.0+
- **Testing**: Soroban CLI + Rust test framework
- **Networks**: Stellar Mainnet, Testnet, Futurenet
- **Tools**: stellar-sdk, soroban-cli

## Project Structure

```
├── ethereum/                           # Ethereum smart contracts
│   ├── contracts/
│   │   ├── core/
│   │   │   ├── PayrollManager.sol     # Main payroll contract
│   │   │   ├── EmployeeRegistry.sol   # Employee management
│   │   │   ├── PaymentProcessor.sol   # Payment execution
│   │   │   └── VestingSchedule.sol    # Token vesting
│   │   ├── governance/
│   │   │   ├── MultiSigWallet.sol     # Multi-signature wallet
│   │   │   ├── Timelock.sol           # Timelock controller
│   │   │   └── GovernanceToken.sol    # Governance token
│   │   ├── payment/
│   │   │   ├── RecurringPayment.sol   # Automated recurring payments
│   │   │   ├── MilestonePayment.sol   # Milestone-based payments
│   │   │   ├── EscrowPayment.sol      # Escrow functionality
│   │   │   └── BatchPayment.sol       # Batch payment processor
│   │   ├── stablecoin/
│   │   │   ├── StablecoinManager.sol  # Stablecoin interface
│   │   │   └── PriceOracle.sol        # Price feed integration
│   │   ├── utils/
│   │   │   ├── ReentrancyGuard.sol    # Security utilities
│   │   │   └── Pausable.sol           # Emergency pause
│   │   └── interfaces/
│   │       ├── IPayrollManager.sol
│   │       ├── IERC20Extended.sol
│   │       └── IPriceOracle.sol
│   ├── test/
│   │   ├── PayrollManager.test.ts
│   │   ├── MilestonePayment.test.ts
│   │   └── MultiSigWallet.test.ts
│   ├── scripts/
│   │   ├── deploy.ts
│   │   ├── upgrade.ts
│   │   └── verify.ts
│   ├── hardhat.config.ts
│   └── package.json
│
├── stellar/                            # Stellar smart contracts (Soroban)
│   ├── contracts/
│   │   ├── payroll/
│   │   │   ├── src/
│   │   │   │   ├── lib.rs             # Main payroll contract
│   │   │   │   ├── storage.rs         # Storage definitions
│   │   │   │   ├── types.rs           # Custom types
│   │   │   │   └── test.rs            # Contract tests
│   │   │   └── Cargo.toml
│   │   ├── milestone/
│   │   │   ├── src/
│   │   │   │   ├── lib.rs             # Milestone payment contract
│   │   │   │   └── test.rs
│   │   │   └── Cargo.toml
│   │   ├── escrow/
│   │   │   ├── src/
│   │   │   │   ├── lib.rs             # Escrow contract
│   │   │   │   └── test.rs
│   │   │   └── Cargo.toml
│   │   └── multisig/
│   │       ├── src/
│   │       │   ├── lib.rs             # Multi-sig wallet
│   │       │   └── test.rs
│   │       └── Cargo.toml
│   ├── scripts/
│   │   ├── deploy.sh
│   │   └── initialize.sh
│   └── Cargo.toml
│
├── docs/
│   ├── architecture.md
│   ├── security.md
│   └── integration-guide.md
├── audits/                             # Security audit reports
└── README.md
```
)

## Installation

### Ethereum Setup

```bash
# Clone repository
git clone https://github.com/SafeVault/vestroll-contract.git
cd vestroll-contracts/solidity-contract

# Install dependencies
npm install

# Copy environment variables
cp .env.example .env

# Compile contracts
npx hardhat compile

# Run tests
npx hardhat test

# Deploy to testnet
npx hardhat run scripts/deploy.ts --network sepolia
```

### Stellar Setup

```bash
# Navigate to stellar directory
cd vestroll-contracts/soroban-contract

# Install Soroban CLI
cargo install --locked soroban-cli --features opt

# Install dependencies
cargo build

# Run tests
cargo test

# Build contracts
soroban contract build


```
