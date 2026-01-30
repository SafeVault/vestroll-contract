#![no_std]
use soroban_sdk::{Address, Symbol, contracterror, contracttype, symbol_short};

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
#[derive(Clone)]
pub enum DataKey {
    Initialized,
    Admin,
    Paused,
    ProtocolAsset,
    AssetWhitelist(Address),
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
}

// Events
pub const PAUSED: Symbol = symbol_short!("paused");
pub const UNPAUSED: Symbol = symbol_short!("unpaused");