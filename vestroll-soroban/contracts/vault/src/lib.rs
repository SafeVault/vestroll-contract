#![no_std]
mod test_vault;
use soroban_sdk::{contract, contractimpl, token, Address, Env};
use vestroll_common::{DataKey, TreasuryStats, VaultError, PAUSED, UNPAUSED};

#[contract]
pub struct VaultContract;

#[contractimpl]
impl VaultContract {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn deposit(
        env: Env,
        from: Address,
        amount: i128,
        asset: Address,
    ) -> Result<(), VaultError> {
        from.require_auth();
        if Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        // Transfer funds to vault
        token::Client::new(&env, &asset).transfer(&from, &env.current_contract_address(), &amount);

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

    pub fn withdraw(env: Env, to: Address, amount: i128, asset: Address) -> Result<(), VaultError> {
        // Implementation for payouts
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(VaultError::AdminNotSet)?;
        admin.require_auth();

        if Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
        };

        let key_locked = DataKey::TotalLocked(asset.clone());
        let mut locked: i128 = env.storage().persistent().get(&key_locked).unwrap_or(0);

        // Cannot withdraw more than locked (reserved) using this function
        if amount > locked {
            // Panic or error
            panic!("Cannot withdraw more than locked amount using release function");
        }

        token::Client::new(&env, &asset).transfer(&env.current_contract_address(), &to, &amount);

        locked -= amount;
        env.storage().persistent().set(&key_locked, &locked);

        let key_deposits = DataKey::TotalDeposits(asset.clone());
        let mut deposits: i128 = env.storage().persistent().get(&key_deposits).unwrap_or(0);
        deposits -= amount;
        if deposits < 0 {
            deposits = 0;
        }
        env.storage().persistent().set(&key_deposits, &deposits);

        Ok(())
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

        let client = token::Client::new(&env, &asset);
        let balance = client.balance(&env.current_contract_address());
        let locked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalLocked(asset.clone()))
            .unwrap_or(0);

        // Available liquidity = Balance - Locked
        let available = balance - locked;

        if amount > available {
            panic!("Insufficient unallocated funds");
        }

        client.transfer(&env.current_contract_address(), &to, &amount);

        Ok(())
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

        if paused && Self::fail_if_paused(&env) {
            return Err(VaultError::ContractPaused);
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
        let is_paused = env
            .storage()
            .instance()
            .get(&DataKey::Paused)
            .unwrap_or(false);
        return is_paused;
    }
}
