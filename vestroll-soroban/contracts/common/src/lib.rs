#![no_std]
use soroban_sdk::{contracttype, Address};

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
