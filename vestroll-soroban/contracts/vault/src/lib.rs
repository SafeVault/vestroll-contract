#![no_std]
mod test_vault;

use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};
use vestroll_common::{
    DataKey, Payment, PayoutEntry, TreasuryStats, VaultError, BATCH_DONE, PAUSED, PAYOUT, UNPAUSED,
};

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    // ====================================================================
    // Initialization
    // ====================================================================

    pub fn initialize(env: Env, admin: Address, token: Address) -> Result<(), VaultError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(VaultError::NotAuthorized); 
        }
        
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Paused, &false);
        
        // Auto-whitelist the primary token
        Self::internal_whitelist_asset(&env, token, true);
        
        Ok(())
    }

    // ====================================================================
    // Deposit
    // ====================================================================

    pub fn deposit(
        env: Env,
        from: Address,
        amount: i128,
        asset: Address,
    ) -> Result<(), VaultError> {
        from.require_auth();
        
        if Self::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        // Use the internal helper to handle whitelist and transfer logic
        Self::internal_transfer_from(&env, &asset, &from, amount)?;

        // Update stats
        let key_deposits = DataKey::TotalDeposits(asset.clone());
        let mut deposits: i128 = env.storage().persistent().get(&key_deposits).unwrap_or(0);
        deposits += amount;
        env.storage().persistent().set(&key_deposits, &deposits);

        let key_locked = DataKey::TotalLocked(asset.clone());
        let mut locked: i128 = env.storage().persistent().get(&key_locked).unwrap_or(0);
        locked += amount;
        env.storage().persistent().set(&key_locked, &locked);

        Ok(())
    }

    // ====================================================================
    // Batch Payouts
    // ====================================================================

    pub fn execute_payouts(
        env: Env,
        vault: Address,
        list: Vec<PayoutEntry>,
    ) -> Result<u32, VaultError> {
        let admin = Self::get_admin_internal(&env)?;
        admin.require_auth();

        if vault != env.current_contract_address() {
            return Err(VaultError::NotAuthorized);
        }

        if Self::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        if list.is_empty() {
            return Err(VaultError::BatchEmptyList);
        }

        let mut processed: u32 = 0;
        for entry in list.iter() {
            let PayoutEntry { recipient, amount, asset } = entry;

            let key_locked = DataKey::TotalLocked(asset.clone());
            let mut locked: i128 = env.storage().persistent().get(&key_locked).unwrap_or(0);

            if amount > locked {
                return Err(VaultError::InsufficientLockedFunds);
            }

            Self::internal_transfer(&env, &asset, &recipient, amount)?;

            // Update Storage
            locked -= amount;
            env.storage().persistent().set(&key_locked, &locked);

            let key_deposits = DataKey::TotalDeposits(asset.clone());
            let mut deposits: i128 = env.storage().persistent().get(&key_deposits).unwrap_or(0);
            deposits = deposits.saturating_sub(amount);
            env.storage().persistent().set(&key_deposits, &deposits);

            env.events().publish((PAYOUT, recipient.clone()), (asset.clone(), amount));
            processed += 1;
        }

        env.events().publish((BATCH_DONE, admin), processed);
        Ok(processed)
    }

    // ====================================================================
    // Withdraw Operations
    // ====================================================================

    pub fn withdraw(env: Env, to: Address, amount: i128, asset: Address) -> Result<(), VaultError> {
        let admin = Self::get_admin_internal(&env)?;
        admin.require_auth();

        if Self::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        Self::ensure_trustline_exists(&env, &to, &asset)?;

        let key_locked = DataKey::TotalLocked(asset.clone());
        let mut locked: i128 = env.storage().persistent().get(&key_locked).unwrap_or(0);

        if amount > locked {
            return Err(VaultError::InsufficientLockedFunds);
        }

        Self::internal_transfer(&env, &asset, &to, amount)?;

        // Update stats
        locked -= amount;
        env.storage().persistent().set(&key_locked, &locked);

        let key_deposits = DataKey::TotalDeposits(asset.clone());
        let mut deposits: i128 = env.storage().persistent().get(&key_deposits).unwrap_or(0);
        deposits = deposits.saturating_sub(amount);
        env.storage().persistent().set(&key_deposits, &deposits);

        Ok(())
    }

    pub fn withdraw_available(
        env: Env,
        to: Address,
        amount: i128,
        asset: Address,
    ) -> Result<(), VaultError> {
        let admin = Self::get_admin_internal(&env)?;
        admin.require_auth();

        if Self::is_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let client = token::Client::new(&env, &asset);
        let balance = client.balance(&env.current_contract_address());
        let locked: i128 = env.storage().persistent().get(&DataKey::TotalLocked(asset.clone())).unwrap_or(0);

        let available = balance - locked;

        if amount > available {
            return Err(VaultError::InsufficientBalance);
        }

        Self::internal_transfer(&env, &asset, &to, amount)
    }

    pub fn set_protocol_asset(env: Env, admin: Address, asset: Address) -> Result<(), VaultError> {
        Self::check_admin(&env, &admin)?;
        env.storage()
            .instance()
            .set(&DataKey::ProtocolAsset, &asset);
        // Auto-whitelist protocol asset
        Self::internal_whitelist_asset(&env, asset, true);
        Ok(())
    }
  
    // ====================================================================
    // Asset & Admin Management
    // ====================================================================

    pub fn set_pause(env: Env, admin: Address, paused: bool) -> Result<bool, VaultError> {
        admin.require_auth();
        let stored_admin = Self::get_admin_internal(&env)?;

        if stored_admin != admin {
            return Err(VaultError::NotAuthorized);
        }

        env.storage().instance().set(&DataKey::Paused, &paused);

        env.events().publish(
            if paused { (PAUSED, admin) } else { (UNPAUSED, admin) },
            env.ledger().timestamp(),
        );
        Ok(true)
    }

    pub fn whitelist_asset(
        env: Env,
        admin: Address,
        asset: Address,
        allowed: bool,
    ) -> Result<(), VaultError> {
        Self::check_admin(&env, &admin)?;
        Self::internal_whitelist_asset(&env, asset, allowed);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, VaultError> {
        Self::get_admin_internal(&env)
    }

    pub fn get_treasury_stats(env: Env, asset: Address) -> TreasuryStats {
        let deposits: i128 = env.storage().persistent().get(&DataKey::TotalDeposits(asset.clone())).unwrap_or(0);
        let locked: i128 = env.storage().persistent().get(&DataKey::TotalLocked(asset.clone())).unwrap_or(0);
        let fees: i128 = env.storage().persistent().get(&DataKey::TotalFees(asset.clone())).unwrap_or(0);

        let client = token::Client::new(&env, &asset);
        let balance = client.balance(&env.current_contract_address());
        let liquidity = balance.saturating_sub(locked);

        TreasuryStats {
            total_deposits: deposits,
            total_locked: locked,
            total_fees: fees,
            total_liquidity: liquidity,
        }
    }

    // ====================================================================
    // Internal Helpers
    // ====================================================================

    pub fn is_paused(env: &Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }

    fn get_admin_internal(env: &Env) -> Result<Address, VaultError> {
        env.storage().instance().get(&DataKey::Admin).ok_or(VaultError::AdminNotSet)
    }

    fn check_admin(env: &Env, admin: &Address) -> Result<(), VaultError> {
        admin.require_auth();
        let stored_admin = Self::get_admin_internal(env)?;
        if admin != &stored_admin {
            return Err(VaultError::NotAuthorized);
        }
        Ok(())
    }

    fn is_whitelisted(env: &Env, asset: &Address) -> bool {
        env.storage().persistent().has(&DataKey::AssetWhitelist(asset.clone()))
    }

    fn internal_whitelist_asset(env: &Env, asset: Address, allowed: bool) {
        if allowed {
            env.storage()
                .persistent()
                .set(&DataKey::AssetWhitelist(asset), &true);
        } else {
            env.storage()
                .persistent()
                .remove(&DataKey::AssetWhitelist(asset));
        }
    }

    fn ensure_trustline_exists(env: &Env, recipient: &Address, asset: &Address) -> Result<(), VaultError> {
        let token_client = token::Client::new(env, asset);
        match token_client.try_balance(recipient) {
            Ok(_) => Ok(()),
            Err(_) => Err(VaultError::TransferFailed), 
        }
    }

    fn internal_transfer(env: &Env, token: &Address, to: &Address, amount: i128) -> Result<(), VaultError> {
        if !Self::is_whitelisted(env, token) { return Err(VaultError::AssetNotWhitelisted); }
        
        let client = token::Client::new(env, token);
        client.transfer(&env.current_contract_address(), to, &amount);
        Ok(())
    }

    fn internal_transfer_from(env: &Env, token: &Address, from: &Address, amount: i128) -> Result<(), VaultError> {
        if !Self::is_whitelisted(env, token) { return Err(VaultError::AssetNotWhitelisted); }
        
        let client = token::Client::new(env, token);
        client.transfer(from, &env.current_contract_address(), &amount);
        Ok(())
    }
}