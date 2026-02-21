#![no_std]
mod test_vault;
use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};
use vestroll_common::{
    DataKey, PayoutEntry, TreasuryStats, VaultError, BATCH_DONE, PAUSED, PAYOUT, UNPAUSED,
};

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    /// Initializes the vault with an admin and the USDC token address.
    pub fn initialize(env: Env, admin: Address, token: Address) -> Result<(), VaultError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(VaultError::NotAuthorized); // Already initialized
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Token, &token);
        Ok(())
    }

    /// Deposits USDC into the vault.
    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<(), VaultError> {
        from.require_auth();
        if Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(VaultError::AdminNotSet)?;
        let client = token::Client::new(&env, &token_addr);

        client.transfer(&from, &env.current_contract_address(), &amount);

        // Update stats
        let key_deposits = DataKey::TotalDeposits(token_addr.clone());
        let mut deposits: i128 = env.storage().persistent().get(&key_deposits).unwrap_or(0);
        deposits += amount;
        env.storage().persistent().set(&key_deposits, &deposits);

        Ok(())
    }

    /// Withdraws USDC from the vault to a specified address. Only admin can call this.
    pub fn withdraw(env: Env, to: Address, amount: i128) -> Result<(), VaultError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VaultError::AdminNotSet)?;
        admin.require_auth();

        if Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(VaultError::AdminNotSet)?;
        let client = token::Client::new(&env, &token_addr);

        let balance = client.balance(&env.current_contract_address());
        if amount > balance {
            return Err(VaultError::InsufficientBalance);
        }

        client.transfer(&env.current_contract_address(), &to, &amount);

        // Update stats
        let key_deposits = DataKey::TotalDeposits(token_addr.clone());
        let mut deposits: i128 = env.storage().persistent().get(&key_deposits).unwrap_or(0);
        deposits -= amount;
        if deposits < 0 {
            deposits = 0;
        }
        env.storage().persistent().set(&key_deposits, &deposits);

        Ok(())
    }

    /// Transfers USDC to another contract address. Only admin can call this.
    pub fn transfer_to_contract(env: Env, to: Address, amount: i128) -> Result<(), VaultError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VaultError::AdminNotSet)?;
        admin.require_auth();

        if Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let token_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(VaultError::AdminNotSet)?;
        let client = token::Client::new(&env, &token_addr);

        let balance = client.balance(&env.current_contract_address());
        if amount > balance {
            return Err(VaultError::InsufficientBalance);
        }

        client.transfer(&env.current_contract_address(), &to, &amount);

        Ok(())
    }

    /// Executes a batch of payouts. Only admin can call this.
    pub fn execute_payouts(
        env: Env,
        vault: Address,
        list: Vec<PayoutEntry>,
    ) -> Result<u32, VaultError> {
        // 1. Auth check
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VaultError::AdminNotSet)?;
        admin.require_auth();

        // 2. Vault address must match this contract
        if vault != env.current_contract_address() {
            return Err(VaultError::NotAuthorized);
        }

        // 3. Block if paused
        if Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        }

        // 4. Reject empty list
        if list.is_empty() {
            return Err(VaultError::BatchEmptyList);
        }

        // 5. Process each payment
        let mut processed: u32 = 0;

        for entry in list.iter() {
            let PayoutEntry {
                recipient,
                amount,
                asset,
            } = entry;

            if amount <= 0 {
                return Err(VaultError::InvalidAmount);
            }

            let client = token::Client::new(&env, &asset);
            let balance = client.balance(&env.current_contract_address());

            if amount > balance {
                return Err(VaultError::InsufficientBalance);
            }

            client.transfer(&env.current_contract_address(), &recipient, &amount);

            // Update deposits if it's the main token OR any asset
            // Actually, main used DataKey::TotalDeposits(asset.clone())
            let key_deposits = DataKey::TotalDeposits(asset.clone());
            let mut deposits: i128 = env.storage().persistent().get(&key_deposits).unwrap_or(0);
            deposits -= amount;
            if deposits < 0 {
                deposits = 0;
            }
            env.storage().persistent().set(&key_deposits, &deposits);

            // Emit one event per payment
            env.events()
                .publish((PAYOUT, recipient.clone()), (asset.clone(), amount));

            processed += 1;
        }

        // Emit batch summary event
        env.events().publish((BATCH_DONE, admin), processed);

        Ok(processed)
    }

    pub fn withdraw_available(
        env: Env,
        to: Address,
        amount: i128,
        asset: Address,
    ) -> Result<(), VaultError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VaultError::AdminNotSet)?;
        admin.require_auth();

        if Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let client = token::Client::new(&env, &asset);
        let balance = client.balance(&env.current_contract_address());
        let locked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalLocked(asset.clone()))
            .unwrap_or(0);

        let available = balance - locked;

        if amount > available {
            // Main used panic, but better return error
            return Err(VaultError::InsufficientBalance);
        }

        client.transfer(&env.current_contract_address(), &to, &amount);

        Ok(())
    }

    pub fn set_pause(env: Env, admin: Address, paused: bool) -> Result<bool, VaultError> {
        admin.require_auth();
        let pause_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VaultError::AdminNotSet)?;

        if pause_admin != admin {
            return Err(VaultError::NotAuthorized);
        }

        env.storage().instance().set(&DataKey::Paused, &paused);

        env.events().publish(
            if paused {
                (PAUSED, admin)
            } else {
                (UNPAUSED, admin)
            },
            env.ledger().timestamp(),
        );
        Ok(true)
    }

    fn fail_if_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false)
    }

    pub fn get_admin(env: Env) -> Result<Address, VaultError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VaultError::AdminNotSet)
    }

    pub fn get_token(env: Env) -> Result<Address, VaultError> {
        env.storage()
            .instance()
            .get(&DataKey::Token)
            .ok_or(VaultError::AdminNotSet)
    }

    pub fn get_treasury_stats(env: Env, asset: Address) -> TreasuryStats {
        let deposits: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalDeposits(asset.clone()))
            .unwrap_or(0);
        let locked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalLocked(asset.clone()))
            .unwrap_or(0);
        let fees: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalFees(asset.clone()))
            .unwrap_or(0);

        let client = token::Client::new(&env, &asset);
        let balance = client.balance(&env.current_contract_address());

        let liquidity = balance - locked;

        TreasuryStats {
            total_deposits: deposits,
            total_locked: locked,
            total_fees: fees,
            total_liquidity: liquidity,
        }
    }
}
