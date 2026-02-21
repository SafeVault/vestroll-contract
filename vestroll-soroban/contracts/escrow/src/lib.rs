#![no_std]

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token, Address, Env};
use vestroll_common::VaultError; // Reuse the VaultError for common vault behaviors

#[contract]
pub struct EscrowContract;

#[derive(Clone)]
#[soroban_sdk::contracttype]
pub enum DataKey {
    Admin,
    ProtocolAsset,
    TotalFunded,
    TotalReleased,
}

#[contractimpl]
impl EscrowContract {
    /// Initializes the Escrow contract with an admin and a protocol asset (e.g. USDC).
    pub fn initialize(env: Env, admin: Address, asset: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ProtocolAsset, &asset);
        env.storage().instance().set(&DataKey::TotalFunded, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalReleased, &0i128);
    }

    /// Fetches the admin address
    pub fn admin(env: Env) -> Result<Address, VaultError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VaultError::AdminNotSet)
    }

    /// Fetches the protocol asset for funding
    pub fn asset(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::ProtocolAsset)
            .expect("Asset not initialized")
    }

    /// Returns the total amount funded into this escrow contract.
    pub fn total_funded(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalFunded)
            .unwrap_or(0i128)
    }

    /// Returns the total amount released to milestone recipients.
    pub fn released_amount(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalReleased)
            .unwrap_or(0i128)
    }

    /// Funder calls this to lock funds in the escrow contract.
    pub fn fund(env: Env, funder: Address, amount: i128) -> Result<(), VaultError> {
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }
        funder.require_auth();

        let asset: Address = Self::asset(env.clone());
        let client = token::Client::new(&env, &asset);

        // Transfer funds from the funder to the contract
        client.transfer_from(
            &env.current_contract_address(),
            &funder,
            &env.current_contract_address(),
            &amount,
        );

        // Update total_funded
        let mut total_funded = Self::total_funded(env.clone());
        total_funded += amount;
        env.storage()
            .instance()
            .set(&DataKey::TotalFunded, &total_funded);

        Ok(())
    }

    /// Admin calls this to release a specific milestone's funds to a recipient.
    pub fn release_milestone(
        env: Env,
        _milestone_id: u32,
        recipient: Address,
        amount: i128,
    ) -> Result<(), VaultError> {
        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let admin: Address = Self::admin(env.clone())?;
        admin.require_auth();

        let asset: Address = Self::asset(env.clone());
        let client = token::Client::new(&env, &asset);

        // Ensure contract has sufficient balance
        let balance = client.balance(&env.current_contract_address());
        if amount > balance {
            return Err(VaultError::InsufficientBalance);
        }

        // Transfer funds from contract to recipient
        client.transfer(&env.current_contract_address(), &recipient, &amount);

        // Update total_released amount
        let mut total_released = Self::released_amount(env.clone());
        total_released += amount;
        env.storage()
            .instance()
            .set(&DataKey::TotalReleased, &total_released);

        Ok(())
    }
}
