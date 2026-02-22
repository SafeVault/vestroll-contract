#![no_std]
use soroban_sdk::{contracterror, contracttype, symbol_short, Address, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractType {
    FixedRate,
    Milestone,
    PayAsYouGo,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractStatus {
    Draft,
    Active,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractMetadata {
    pub employer: Address,
    pub employee: Address,
    pub contract_type: ContractType,
    pub status: ContractStatus,
    pub amount: i128,
    pub asset: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreasuryStats {
    pub total_deposits: i128,
    pub total_locked: i128,
    pub total_fees: i128,
    pub total_liquidity: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayoutEntry {
    pub recipient: Address,
    pub amount: i128,
    pub asset: Address,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Initialized,
    Admin,
    Paused,
    ProtocolAsset,
    AssetWhitelist(Address),
    TotalDeposits(Address),
    TotalLocked(Address),
    TotalFees(Address),
    Token,
}

// Error
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum VaultError {
    AdminNotSet = 1,
    NotAuthorized = 2,
    ContractPaused = 3,
    AssetNotWhitelisted = 4,
    InvalidAmount = 5,
    SelfTransfer = 6,
    TransferFailed = 7,
    AssetAlreadyWhitelisted = 8,
    AssetNotProtocol = 9,
    InsufficientBalance = 10,
    InsufficientLockedFunds = 11,
    MissingTrustline = 12,
    BatchEmptyList = 13,
    BatchPayoutFailed = 14,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum LifecycleError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ContractNotFound = 3,
    NotAuthorized = 4,
    ContractNotActive = 5,
    InvalidContractType = 6,
    InvalidMilestoneData = 7,
    MilestoneNotFound = 8,
    MilestoneAlreadyCompleted = 9,
    MilestoneNotCompleted = 10,
    InsufficientContractFunds = 11,
    ProfileContractNotSet = 12,
    VaultContractNotSet = 13,
    EmployeeCannotReceivePayment = 14,
    EmployeeProfileNotFound = 15,
    VaultPaymentFailed = 16,
}



// Events
pub const PAUSED: Symbol = symbol_short!("paused");
pub const UNPAUSED: Symbol = symbol_short!("unpaused");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Payment {
    pub recipient: Address,
    pub amount: i128,
}
pub const PAYOUT: Symbol = symbol_short!("payout");
pub const BATCH_DONE: Symbol = symbol_short!("batch");
