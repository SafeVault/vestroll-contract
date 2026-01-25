#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, Address, Env, Symbol,
    Vec,
};

// Storage keys as per your format
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    ProposedAdmin,
    Managers,
    Initialized,
    ManagerList,
}

// Error types as per your format
#[derive(Clone, Copy)]
#[contracterror]
pub enum ContractError {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    ManagerAlreadyExists = 4,
    ManagerNotFound = 5,
    InvalidAddress = 6,
    TransferToSelf = 7,
    NotProposedAdmin = 8,
    Unauthorized = 9,
}

// Events - using your contracttype format
#[contracttype]
pub struct InitializedEvent {
    pub admin: Address,
}

#[contracttype]
pub struct AdminProposedEvent {
    pub current_admin: Address,
    pub proposed_admin: Address,
}

#[contracttype]
pub struct AdminChangedEvent {
    pub previous_admin: Address,
    pub new_admin: Address,
}

#[contracttype]
pub struct ManagerAddedEvent {
    pub manager: Address,
    pub admin: Address,
}

#[contracttype]
pub struct ManagerRemovedEvent {
    pub manager: Address,
    pub admin: Address,
}

#[contract]
pub struct IdentityManagementContract;

#[contractimpl]
impl IdentityManagementContract {
    /// Initialize the contract with a root admin
    /// Can only be called once
    pub fn initialize(env: Env, admin: Address) -> Result<(), ContractError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Initialized, &true);

        let empty_managers: Vec<Address> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::ManagerList, &empty_managers);

        env.events().publish(
            (Symbol::new(&env, "initialized"),),
            InitializedEvent {
                admin: admin.clone(),
            },
        );

        Ok(())
    }

    /// Add a manager to the whitelist (Admin only)
    pub fn add_manager(env: Env, caller: Address, manager: Address) -> Result<(), ContractError> {
        Self::require_initialized(&env)?;
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            panic_with_error!(&env, ContractError::NotAuthorized);
        }

        let mut managers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::ManagerList)
            .unwrap_or(Vec::new(&env));

        if managers.contains(&manager) {
            panic_with_error!(&env, ContractError::ManagerAlreadyExists);
        }

        managers.push_back(manager.clone());
        env.storage()
            .persistent()
            .set(&DataKey::ManagerList, &managers);

        env.events().publish(
            (Symbol::new(&env, "manager_added"),),
            ManagerAddedEvent {
                manager: manager.clone(),
                admin: admin.clone(),
            },
        );

        Ok(())
    }

    /// Remove a manager from the whitelist (Admin only)
    pub fn remove_manager(
        env: Env,
        caller: Address,
        manager: Address,
    ) -> Result<(), ContractError> {
        Self::require_initialized(&env)?;
        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            panic_with_error!(&env, ContractError::NotAuthorized);
        }

        let mut managers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::ManagerList)
            .unwrap_or(Vec::new(&env));

        let index = managers.iter().position(|m| m == manager);
        if index.is_none() {
            panic_with_error!(&env, ContractError::ManagerNotFound);
        }

        managers.remove(index.unwrap() as u32);
        env.storage()
            .persistent()
            .set(&DataKey::ManagerList, &managers);

        env.events().publish(
            (Symbol::new(&env, "manager_removed"),),
            ManagerRemovedEvent {
                manager: manager.clone(),
                admin: admin.clone(),
            },
        );

        Ok(())
    }

    /// Propose a new admin (Admin only)
    pub fn propose_admin(
        env: Env,
        caller: Address,
        proposed_admin: Address,
    ) -> Result<(), ContractError> {
        Self::require_initialized(&env)?;

        caller.require_auth();

        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        if caller != admin {
            panic_with_error!(&env, ContractError::NotAuthorized);
        }

        if proposed_admin == admin {
            panic_with_error!(&env, ContractError::TransferToSelf);
        }

        env.storage()
            .instance()
            .set(&DataKey::ProposedAdmin, &proposed_admin);

        env.events().publish(
            (Symbol::new(&env, "admin_proposed"),),
            AdminProposedEvent {
                current_admin: admin.clone(),
                proposed_admin: proposed_admin.clone(),
            },
        );

        Ok(())
    }

    /// Claim admin role (Proposed admin only)
    pub fn claim_admin(env: Env, caller: Address) -> Result<(), ContractError> {
        Self::require_initialized(&env)?;

        caller.require_auth();

        let proposed_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::ProposedAdmin)
            .ok_or(ContractError::NotProposedAdmin)?;

        if caller != proposed_admin {
            panic_with_error!(&env, ContractError::NotProposedAdmin);
        }

        let current_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)?;

        env.storage()
            .instance()
            .set(&DataKey::Admin, &proposed_admin);

        env.storage().instance().remove(&DataKey::ProposedAdmin);

        env.events().publish(
            (Symbol::new(&env, "admin_changed"),),
            AdminChangedEvent {
                previous_admin: current_admin,
                new_admin: proposed_admin.clone(),
            },
        );

        Ok(())
    }

    /// Get current admin
    pub fn get_admin(env: Env) -> Result<Address, ContractError> {
        Self::require_initialized(&env)?;

        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ContractError::NotInitialized)
    }

    /// Get proposed admin (if any)
    pub fn get_proposed_admin(env: Env) -> Result<Option<Address>, ContractError> {
        Self::require_initialized(&env)?;

        Ok(env.storage().instance().get(&DataKey::ProposedAdmin))
    }

    /// Check if an address is a manager
    pub fn is_manager(env: Env, address: Address) -> Result<bool, ContractError> {
        Self::require_initialized(&env)?;

        let managers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::ManagerList)
            .unwrap_or(Vec::new(&env));

        Ok(managers.contains(&address))
    }

    /// Get all managers
    pub fn get_managers(env: Env) -> Result<Vec<Address>, ContractError> {
        Self::require_initialized(&env)?;

        Ok(env
            .storage()
            .persistent()
            .get(&DataKey::ManagerList)
            .unwrap_or(Vec::new(&env)))
    }

    /// Get total number of managers
    pub fn get_manager_count(env: Env) -> Result<u32, ContractError> {
        Self::require_initialized(&env)?;

        let managers: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::ManagerList)
            .unwrap_or(Vec::new(&env));

        Ok(managers.len())
    }

    pub fn is_initialized(env: Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    fn require_initialized(env: &Env) -> Result<(), ContractError> {
        if !env.storage().instance().has(&DataKey::Initialized) {
            panic_with_error!(env, ContractError::NotInitialized);
        }
        Ok(())
    }
}
