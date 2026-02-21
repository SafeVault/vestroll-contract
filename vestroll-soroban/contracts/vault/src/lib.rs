#![no_std]
mod test_vault;
use soroban_sdk::{contract, contractimpl, token, Address, Env};
use vestroll_common::{DataKey, VaultError, PAUSED, UNPAUSED};

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
    pub fn deposit(
        env: Env,
        from: Address,
        amount: i128,
    ) -> Result<(), VaultError> {
        from.require_auth();
        if Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        if amount <= 0 {
            return Err(VaultError::InvalidAmount);
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).ok_or(VaultError::AdminNotSet)?;
        let client = token::Client::new(&env, &token_addr);

        client.transfer(&from, &env.current_contract_address(), &amount);

        // Update stats (optional but good for tracking)
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

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).ok_or(VaultError::AdminNotSet)?;
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
        if deposits < 0 { deposits = 0; }
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

        let token_addr: Address = env.storage().instance().get(&DataKey::Token).ok_or(VaultError::AdminNotSet)?;
        let client = token::Client::new(&env, &token_addr);

        let balance = client.balance(&env.current_contract_address());
        if amount > balance {
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
            .ok_or_else(|| return VaultError::AdminNotSet)?;

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
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }

    pub fn get_admin(env: Env) -> Result<Address, VaultError> {
        env.storage().instance().get(&DataKey::Admin).ok_or(VaultError::AdminNotSet)
    }

    pub fn get_token(env: Env) -> Result<Address, VaultError> {
        env.storage().instance().get(&DataKey::Token).ok_or(VaultError::AdminNotSet)
    }
}
