#![no_std]

mod errors;
mod storage;
mod types;
#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec, Map, symbol_short, Symbol};

use crate::errors::ContractError;
use crate::storage::{DataKey, WorkerWallet};
use crate::types::{Profile, ProfileType};

// ========================================================================
// Contract
// ========================================================================

#[contract]
pub struct ProfileContract;

#[contractimpl]
impl ProfileContract {
    // ====================================================================
    // Initialization
    // ====================================================================

    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(ContractError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);

        Ok(())
    }

    // ====================================================================
    // Profile Management
    // ====================================================================

    pub fn create_profile(
        env: Env,
        user: Address,
        name: String,
        is_org: bool,
    ) -> Result<Profile, ContractError> {
        if !Self::is_initialized(env.clone()) {
            return Err(ContractError::NotInitialized);
        }

        if Self::has_profile(env.clone(), user.clone()) {
            return Err(ContractError::WalletAlreadyRegistered);
        }

        let profile_type = if is_org {
            ProfileType::Organization
        } else {
            ProfileType::Worker
        };

        let profile = Profile {
            id: user.clone(),
            name,
            profile_type,
            created_at: env.ledger().timestamp(),
            is_active: true,
        };

        env.storage().instance().set(&DataKey::Profile(user), &profile);

        Ok(profile)
    }

    pub fn get_profile(env: Env, user: Address) -> Result<Profile, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Profile(user))
            .ok_or(ContractError::ProfileNotFound)
    }

    pub fn has_profile(env: Env, user: Address) -> bool {
        env.storage().instance().has(&DataKey::Profile(user))
    }

    pub fn deactivate_profile(env: Env, user: Address) -> Result<(), ContractError> {
        user.require_auth();

        if !Self::is_initialized(env.clone()) {
            return Err(ContractError::NotInitialized);
        }

        let mut profile = Self::get_profile(env.clone(), user.clone())?;
        profile.is_active = false;

        env.storage().instance().set(&DataKey::Profile(user), &profile);

        Ok(())
    }

    // ====================================================================
    // Worker Wallet Management
    // ====================================================================

    pub fn register_worker_wallet(
        env: Env,
        worker: Address,
        wallet_address: String,
    ) -> Result<WorkerWallet, ContractError> {
        if !Self::is_initialized(env.clone()) {
            return Err(ContractError::NotInitialized);
        }

        worker.require_auth();

        let profile = Self::get_profile(env.clone(), worker.clone())?;
        if profile.profile_type != ProfileType::Worker {
            return Err(ContractError::NotAWorker);
        }

        if wallet_address.len() != 56 {
            return Err(ContractError::InvalidWalletAddress);
        }

        if Self::has_wallet_registered(env.clone(), worker.clone()) {
            return Err(ContractError::WalletAlreadyRegistered);
        }

        let wallet = WorkerWallet {
            worker_id: worker.clone(),
            wallet_address: wallet_address.clone(),
            trustline_verified: false,
            last_verified: env.ledger().timestamp(),
            is_active: true,
        };

        env.storage().instance().set(&DataKey::WorkerWallet(worker.clone()), &wallet);

        // Trigger initial trustline verification
        Self::verify_trustline(env, worker)?;

        Ok(wallet)
    }

    pub fn get_worker_wallet(env: Env, worker: Address) -> Result<WorkerWallet, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::WorkerWallet(worker))
            .ok_or(ContractError::ProfileNotFound)
    }

    pub fn has_wallet_registered(env: Env, worker: Address) -> bool {
        env.storage().instance().has(&DataKey::WorkerWallet(worker))
    }

    pub fn update_wallet_address(
        env: Env,
        worker: Address,
        new_wallet_address: String,
    ) -> Result<WorkerWallet, ContractError> {
        if !Self::is_initialized(env.clone()) {
            return Err(ContractError::NotInitialized);
        }

        worker.require_auth();

        let mut wallet = Self::get_worker_wallet(env.clone(), worker.clone())?;

        if new_wallet_address.len() != 56 {
            return Err(ContractError::InvalidWalletAddress);
        }

        wallet.wallet_address = new_wallet_address;
        wallet.trustline_verified = false;
        wallet.last_verified = env.ledger().timestamp();

        env.storage().instance().set(&DataKey::WorkerWallet(worker), &wallet);

        Ok(wallet)
    }

    // ====================================================================
    // Trustline Verification
    // ====================================================================

    pub fn verify_trustline(env: Env, worker: Address) -> Result<bool, ContractError> {
        if !Self::is_initialized(env.clone()) {
            return Err(ContractError::NotInitialized);
        }

        let wallet = Self::get_worker_wallet(env.clone(), worker.clone())?;

        // In production, this would call Horizon
        let trustline_exists = wallet.wallet_address.len() == 56;

        let mut updated_wallet = wallet;
        updated_wallet.trustline_verified = trustline_exists;
        updated_wallet.last_verified = env.ledger().timestamp();

        env.storage().instance().set(&DataKey::WorkerWallet(worker), &updated_wallet);

        Ok(trustline_exists)
    }

    pub fn get_trustline_status(env: Env, worker: Address) -> Result<bool, ContractError> {
        if !Self::is_initialized(env.clone()) {
            return Err(ContractError::NotInitialized);
        }

        let wallet = Self::get_worker_wallet(env, worker)?;
        Ok(wallet.trustline_verified)
    }

    pub fn can_receive_payment(env: Env, worker: Address) -> Result<bool, ContractError> {
        if !Self::is_initialized(env.clone()) {
            return Err(ContractError::NotInitialized);
        }

        let wallet = Self::get_worker_wallet(env.clone(), worker.clone())?;

        if !wallet.trustline_verified {
            return Self::verify_trustline(env, worker);
        }

        Ok(true)
    }

    pub fn batch_verify_trustlines(env: Env, workers: Vec<Address>) -> Map<Address, bool> {
        let mut results = Map::new(&env);

        if !Self::is_initialized(env.clone()) {
            return results;
        }

        for i in 0..workers.len() {
            let worker = workers.get(i).unwrap();
            if let Ok(status) = Self::verify_trustline(env.clone(), worker.clone()) {
                results.set(worker, status);
            }
        }

        results
    }

    // ====================================================================
    // Organization Management
    // ====================================================================

    pub fn add_worker_to_organization(
        env: Env,
        organization: Address,
        worker: Address,
    ) -> Result<(), ContractError> {
        if !Self::is_initialized(env.clone()) {
            return Err(ContractError::NotInitialized);
        }

        let org_profile = Self::get_profile(env.clone(), organization.clone())?;
        if org_profile.profile_type != ProfileType::Organization {
            return Err(ContractError::NotAnOrganization);
        }

        let worker_profile = Self::get_profile(env.clone(), worker.clone())?;
        if worker_profile.profile_type != ProfileType::Worker {
            return Err(ContractError::NotAWorker);
        }

        let key = DataKey::OrgWorkers(organization.clone());
        let mut workers: Vec<Address> = env.storage()
            .instance()
            .get(&key)
            .unwrap_or(Vec::new(&env));

        // Check if worker already exists
        let mut exists = false;
        for i in 0..workers.len() {
            if workers.get(i).unwrap() == worker {
                exists = true;
                break;
            }
        }

        if !exists {
            workers.push_back(worker.clone());
            env.storage().instance().set(&key, &workers);

            // Update worker count
            let count_key = DataKey::WorkerCount(organization);
            let count: u32 = env.storage().instance().get(&count_key).unwrap_or(0);
            env.storage().instance().set(&count_key, &(count + 1));
        }

        Ok(())
    }

    pub fn get_organization_workers(env: Env, organization: Address) -> Vec<Address> {
        if !Self::is_initialized(env.clone()) {
            return Vec::new(&env);
        }

        env.storage()
            .instance()
            .get(&DataKey::OrgWorkers(organization))
            .unwrap_or(Vec::new(&env))
    }

    pub fn get_worker_count(env: Env, organization: Address) -> u32 {
        if !Self::is_initialized(env.clone()) {
            return 0;
        }

        env.storage()
            .instance()
            .get(&DataKey::WorkerCount(organization))
            .unwrap_or(0)
    }

    // ====================================================================
    // Getters
    // ====================================================================

    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)
    }

    // ====================================================================
    // Statistics
    // ====================================================================

    pub fn get_profile_stats(env: Env, user: Address) -> Map<Symbol, i128> {
        let mut stats = Map::new(&env);
    
        if !Self::is_initialized(env.clone()) {
            return stats;
        }
    
        if let Ok(profile) = Self::get_profile(env.clone(), user.clone()) {
            let profile_type = profile.profile_type.clone();
            
            stats.set(symbol_short!("type"), (profile_type.clone() as u32) as i128);
            stats.set(symbol_short!("active"), profile.is_active as i128);
            stats.set(symbol_short!("created"), profile.created_at as i128);
    
            if profile_type == ProfileType::Worker {
                if let Ok(wallet) = Self::get_worker_wallet(env.clone(), user) {
                    stats.set(symbol_short!("wallet"), 1i128);
                    stats.set(symbol_short!("trust"), wallet.trustline_verified as i128);
                    stats.set(symbol_short!("verified"), wallet.last_verified as i128);
                } else {
                    stats.set(symbol_short!("wallet"), 0i128);
                }
            } else {
                let count = Self::get_worker_count(env, user);
                stats.set(symbol_short!("workers"), count as i128);
            }
        }
    
        stats
    }
}
