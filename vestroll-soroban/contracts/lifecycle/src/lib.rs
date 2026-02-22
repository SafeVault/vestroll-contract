#![no_std]
mod test_lifecycle;

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec, String, symbol_short};

use vestroll_common::{ContractType, ContractMetadata, LifecycleError, ContractStatus};
use vestroll_profile::{ProfileContractClient};
use vestroll_vault::{VaultContractClient};

#[contract]
pub struct LifecycleContract;

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Initialized,
    NextContractId,
    Contract(u32),
    EmployerContracts(Address),
    EmployeeContracts(Address),
    VaultAddress,
    ProfileAddress,
    ProtocolAsset,
}

#[contracttype]
#[derive(Clone)]
pub struct Contract {
    pub id: u32,
    pub employer: Address,
    pub employee: Address,
    pub contract_type: ContractType,
    pub status: ContractStatus,
    pub total_amount: i128,
    pub paid_amount: i128,
    pub asset: Address,
    pub metadata: ContractMetadata,
    pub created_at: u64,
    pub milestones: Option<Vec<Milestone>>,
}

#[contracttype]
#[derive(Clone)]
pub struct Milestone {
    pub id: u32,
    pub description: String,
    pub amount: i128,
    pub completed: bool,
    pub completed_at: Option<u64>,
}

#[contractimpl]
impl LifecycleContract {
    pub fn initialize(
        env: Env, 
        admin: Address, 
        vault_address: Address, 
        profile_address: Address,
        protocol_asset: Address
    ) -> Result<(), LifecycleError> {
        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(LifecycleError::AlreadyInitialized);
        }

        admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::VaultAddress, &vault_address);
        env.storage().instance().set(&DataKey::ProfileAddress, &profile_address);
        env.storage().instance().set(&DataKey::ProtocolAsset, &protocol_asset);
        env.storage().instance().set(&DataKey::NextContractId, &1u32);
        env.storage().instance().set(&DataKey::Initialized, &true);

        Ok(())
    }

    pub fn create_contract(
        env: Env,
        employer: Address,
        employee: Address,
        contract_type: ContractType,
        total_amount: i128,
        asset: Address,
        metadata: ContractMetadata,
        milestones: Option<Vec<Milestone>>,
    ) -> Result<u32, LifecycleError> {
        employer.require_auth();
        Self::ensure_initialized(&env)?;
        
        match contract_type {
            ContractType::Milestone => {
                if milestones.is_none() { return Err(LifecycleError::InvalidMilestoneData); }
            }
            _ => {
                if milestones.is_some() { return Err(LifecycleError::InvalidContractType); }
            }
        }

        Self::ensure_employee_can_receive_payment(&env, &employee, &asset)?;

        let contract_id = Self::get_next_id(&env);
        let contract = Contract {
            id: contract_id,
            employer: employer.clone(),
            employee: employee.clone(),
            contract_type: contract_type.clone(),
            status: ContractStatus::Active,
            total_amount,
            paid_amount: 0,
            asset: asset.clone(),
            metadata: metadata.clone(),
            created_at: env.ledger().timestamp(),
            milestones,
        };

        env.storage().instance().set(&DataKey::Contract(contract_id), &contract);
        Self::add_to_employer_list(&env, employer.clone(), contract_id);
        Self::add_to_employee_list(&env, employee.clone(), contract_id);
        env.storage().instance().set(&DataKey::NextContractId, &(contract_id + 1));

        env.events().publish(
            (symbol_short!("CREATE"), contract_id),
            (employer, employee, contract_type),
        );

        Ok(contract_id)
    }

    pub fn process_fixed_payment(
        env: Env,
        employer: Address,
        contract_id: u32,
        amount: i128,
    ) -> Result<(), LifecycleError> {
        employer.require_auth();
        Self::ensure_initialized(&env)?;

        let mut contract = Self::get_contract_internal(&env, contract_id)?;
        if contract.employer != employer { return Err(LifecycleError::NotAuthorized); }
        if contract.contract_type != ContractType::FixedRate { return Err(LifecycleError::InvalidContractType); }
        if contract.status != ContractStatus::Active { return Err(LifecycleError::ContractNotActive); }

        let remaining = contract.total_amount - contract.paid_amount;
        if amount > remaining { return Err(LifecycleError::InsufficientContractFunds); }

        Self::ensure_employee_can_receive_payment(&env, &contract.employee, &contract.asset)?;
        Self::process_vault_payment(&env, &contract.employee, amount, &contract.asset)?;

        contract.paid_amount += amount;
        if contract.paid_amount >= contract.total_amount {
            contract.status = ContractStatus::Completed;
        }

        env.storage().instance().set(&DataKey::Contract(contract_id), &contract);
        env.events().publish(
            (symbol_short!("PAYMENT"), contract_id),
            (employer, contract.employee, amount),
        );

        Ok(())
    }

    pub fn complete_milestone(
        env: Env,
        employer: Address,
        contract_id: u32,
        milestone_id: u32,
    ) -> Result<(), LifecycleError> {
        employer.require_auth();
        Self::ensure_initialized(&env)?;

        let mut contract = Self::get_contract_internal(&env, contract_id)?;
        if contract.employer != employer { return Err(LifecycleError::NotAuthorized); }
        if contract.contract_type != ContractType::Milestone { return Err(LifecycleError::InvalidContractType); }
        if contract.status != ContractStatus::Active { return Err(LifecycleError::ContractNotActive); }

        let mut milestones = contract.milestones.ok_or(LifecycleError::InvalidMilestoneData)?;
        let mut found = false;

        for i in 0..milestones.len() {
            let mut milestone = milestones.get(i).unwrap();
            if milestone.id == milestone_id {
                if milestone.completed { return Err(LifecycleError::MilestoneAlreadyCompleted); }
                milestone.completed = true;
                milestone.completed_at = Some(env.ledger().timestamp());
                milestones.set(i, milestone);
                found = true;
                break;
            }
        }

        if !found { return Err(LifecycleError::MilestoneNotFound); }

        contract.milestones = Some(milestones);
        env.storage().instance().set(&DataKey::Contract(contract_id), &contract);
        Ok(())
    }

    pub fn process_milestone_payment(
        env: Env,
        employer: Address,
        contract_id: u32,
        milestone_id: u32,
    ) -> Result<(), LifecycleError> {
        employer.require_auth();
        Self::ensure_initialized(&env)?;

        let mut contract = Self::get_contract_internal(&env, contract_id)?;
        if contract.employer != employer { return Err(LifecycleError::NotAuthorized); }
        if contract.contract_type != ContractType::Milestone { return Err(LifecycleError::InvalidContractType); }
        if contract.status != ContractStatus::Active { return Err(LifecycleError::ContractNotActive); }

        let milestones = contract.milestones.as_ref().ok_or(LifecycleError::InvalidMilestoneData)?;
        let mut milestone_amount = 0;
        let mut milestone_found = false;
        let mut all_completed = true;

        for i in 0..milestones.len() {
            let milestone = milestones.get(i).unwrap();
            if milestone.id == milestone_id {
                if !milestone.completed { return Err(LifecycleError::MilestoneNotCompleted); }
                milestone_amount = milestone.amount;
                milestone_found = true;
            }
            if !milestone.completed { all_completed = false; }
        }

        if !milestone_found { return Err(LifecycleError::MilestoneNotFound); }

        Self::ensure_employee_can_receive_payment(&env, &contract.employee, &contract.asset)?;
        Self::process_vault_payment(&env, &contract.employee, milestone_amount, &contract.asset)?;

        contract.paid_amount += milestone_amount;
        if all_completed && contract.paid_amount >= contract.total_amount {
            contract.status = ContractStatus::Completed;
        }

        env.storage().instance().set(&DataKey::Contract(contract_id), &contract);
        env.events().publish(
            (symbol_short!("MILESTONE"), contract_id, milestone_id),
            (employer, contract.employee, milestone_amount),
        );

        Ok(())
    }

    pub fn process_payg_payment(
        env: Env,
        employer: Address,
        contract_id: u32,
        amount: i128,
    ) -> Result<(), LifecycleError> {
        employer.require_auth();
        Self::ensure_initialized(&env)?;

        let mut contract = Self::get_contract_internal(&env, contract_id)?;
        if contract.employer != employer { return Err(LifecycleError::NotAuthorized); }
        if contract.contract_type != ContractType::PayAsYouGo { return Err(LifecycleError::InvalidContractType); }
        if contract.status != ContractStatus::Active { return Err(LifecycleError::ContractNotActive); }

        let remaining = contract.total_amount - contract.paid_amount;
        if amount > remaining { return Err(LifecycleError::InsufficientContractFunds); }

        Self::ensure_employee_can_receive_payment(&env, &contract.employee, &contract.asset)?;
        Self::process_vault_payment(&env, &contract.employee, amount, &contract.asset)?;

        contract.paid_amount += amount;
        if contract.paid_amount >= contract.total_amount {
            contract.status = ContractStatus::Completed;
        }

        env.storage().instance().set(&DataKey::Contract(contract_id), &contract);
        env.events().publish(
            (symbol_short!("PAYG"), contract_id),
            (employer, contract.employee, amount),
        );

        Ok(())
    }

    pub fn cancel_contract(
        env: Env,
        caller: Address,
        contract_id: u32,
    ) -> Result<(), LifecycleError> {
        caller.require_auth();
        Self::ensure_initialized(&env)?;

        let mut contract = Self::get_contract_internal(&env, contract_id)?;
        if contract.employer != caller {
            let admin = Self::get_admin(&env)?;
            if caller != admin { return Err(LifecycleError::NotAuthorized); }
        }

        if contract.status != ContractStatus::Active { return Err(LifecycleError::ContractNotActive); }

        contract.status = ContractStatus::Cancelled;
        env.storage().instance().set(&DataKey::Contract(contract_id), &contract);

        env.events().publish((symbol_short!("CANCEL"), contract_id), caller);
        Ok(())
    }

    pub fn get_contract(env: Env, id: u32) -> Result<Contract, LifecycleError> {
        Self::ensure_initialized(&env)?;
        Self::get_contract_internal(&env, id)
    }

    pub fn get_employer_contracts(env: Env, employer: Address) -> Vec<u32> {
        if !Self::is_initialized(&env) { return Vec::new(&env); }
        env.storage().instance().get(&DataKey::EmployerContracts(employer)).unwrap_or(Vec::new(&env))
    }

    pub fn get_employee_contracts(env: Env, employee: Address) -> Vec<u32> {
        if !Self::is_initialized(&env) { return Vec::new(&env); }
        env.storage().instance().get(&DataKey::EmployeeContracts(employee)).unwrap_or(Vec::new(&env))
    }

    pub fn get_admin(env: &Env) -> Result<Address, LifecycleError> {
        env.storage().instance().get(&DataKey::Admin).ok_or(LifecycleError::NotInitialized)
    }

    pub fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    // ====================================================================
    // Internal Helpers
    // ====================================================================

    fn ensure_initialized(env: &Env) -> Result<(), LifecycleError> {
        if !Self::is_initialized(env) { return Err(LifecycleError::NotInitialized); }
        Ok(())
    }

    fn get_next_id(env: &Env) -> u32 {
        env.storage().instance().get(&DataKey::NextContractId).unwrap_or(1)
    }

    fn get_contract_internal(env: &Env, id: u32) -> Result<Contract, LifecycleError> {
        env.storage().instance().get(&DataKey::Contract(id)).ok_or(LifecycleError::ContractNotFound)
    }

    fn add_to_employer_list(env: &Env, employer: Address, contract_id: u32) {
        let key = DataKey::EmployerContracts(employer);
        let mut contracts: Vec<u32> = env.storage().instance().get(&key).unwrap_or(Vec::new(env));
        contracts.push_back(contract_id);
        env.storage().instance().set(&key, &contracts);
    }

    fn add_to_employee_list(env: &Env, employee: Address, contract_id: u32) {
        let key = DataKey::EmployeeContracts(employee);
        let mut contracts: Vec<u32> = env.storage().instance().get(&key).unwrap_or(Vec::new(env));
        contracts.push_back(contract_id);
        env.storage().instance().set(&key, &contracts);
    }

    fn ensure_employee_can_receive_payment(
        env: &Env,
        employee: &Address,
        _asset: &Address,
    ) -> Result<(), LifecycleError> {
        let profile_address: Address = env.storage()
            .instance()
            .get(&DataKey::ProfileAddress)
            .ok_or(LifecycleError::ProfileContractNotSet)?;

        let profile_client = ProfileContractClient::new(env, &profile_address);
        
        // Use try_ to get a Result, then unwrap the Soroban Result, then check the bool
        let can_receive = profile_client.try_can_receive_payment(employee)
            .map_err(|_| LifecycleError::EmployeeProfileNotFound)? // Catches transport/contract error
            .map_err(|_| LifecycleError::EmployeeProfileNotFound)?; // Catches conversion error

        if !can_receive {
            return Err(LifecycleError::EmployeeCannotReceivePayment);
        }

        Ok(())
    }

    fn process_vault_payment(
        env: &Env,
        recipient: &Address,
        amount: i128,
        asset: &Address,
    ) -> Result<(), LifecycleError> {
        let vault_address: Address = env.storage()
            .instance()
            .get(&DataKey::VaultAddress)
            .ok_or(LifecycleError::VaultContractNotSet)?;

        let vault_client = VaultContractClient::new(env, &vault_address);
        
        // Use the 'try_' version of the client method to catch contract errors
        vault_client.try_withdraw_available(recipient, &amount, asset)
            .map_err(|_| LifecycleError::VaultPaymentFailed)? // Catches Invocation error
            .map_err(|_| LifecycleError::VaultPaymentFailed)?; // Catches logic error if applicable

        Ok(())
    }
}
