#![no_std]
mod test_vault;
use soroban_sdk::{contract, contractimpl, Address, Env};
use vestroll_common::{DataKey, ContractMetadata, VaultError, PAUSED, UNPAUSED};

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn deposit(env: Env, from: Address, amount: i128, asset: Address) -> Result<(), VaultError> {
        // Implementation for escrowing funds
        if Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        Ok(())
    }

    pub fn withdraw(env: Env, to: Address, amount: i128, asset: Address)  -> Result<(), VaultError> {
        // Implementation for payouts
        if Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        Ok(())
    }

    pub fn set_pause(env: Env, admin: Address, paused: bool) -> Result<bool, VaultError> {
        admin.require_auth();
        let pause_admin: Address = env.storage().instance().get(&DataKey::Admin).ok_or_else(|| return VaultError::AdminNotSet)?;

        if pause_admin != admin {
            return Err(VaultError::NotAuthorized);
        }

        if paused && Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        env.storage().instance().set(&DataKey::Paused, &paused);

        env.events().publish(
            if paused { (PAUSED, admin) } else { (UNPAUSED, admin) },
            env.ledger().timestamp(),
        );
        Ok(true)
    }

    fn fail_if_paused(env: &Env) -> bool {
        let is_paused = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
        return is_paused;
    }
}
