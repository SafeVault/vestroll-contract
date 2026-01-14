#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env};
use vestroll_common::ContractMetadata;

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn deposit(env: Env, from: Address, amout: i128, asset: Address) {
        // Implementation for escrowing funds
    }

    pub fn withdraw(env: Env, to: Address, amount: i128, asset: Address) {
        // Implementation for payouts
    }
}
