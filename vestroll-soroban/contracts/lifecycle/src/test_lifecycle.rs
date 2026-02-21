#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _}, 
    token, Address, Env, String, Vec
};

use vestroll_common::{ContractType, ContractStatus};
use vestroll_profile::{ProfileContract, ProfileContractClient};
use vestroll_vault::{VaultContract, VaultContractClient};

use crate::{LifecycleContract, LifecycleContractClient, Milestone};

fn setup_env() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.ledger().with_mut(|li| li.sequence_number = 1000);
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let employer = Address::generate(&env);
    let employee = Address::generate(&env);
    
    (env, admin, employer, employee)
}

fn setup_contracts(
    env: &Env,
    admin: &Address,
    employer: &Address,
    employee: &Address,
) -> (
    LifecycleContractClient<'static>,
    Address,
    Address,
    Address,
    token::Client<'static>,
) {
    let profile_id = env.register(ProfileContract, ());
    let profile_client = ProfileContractClient::new(env, &profile_id);
    profile_client.initialize(admin);
    
    profile_client.create_profile(employer, &String::from_str(env, "Employer"), &true);
    profile_client.create_profile(employee, &String::from_str(env, "Employee"), &false);
    
    let wallet = String::from_str(env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF");
    profile_client.register_worker_wallet(employee, &wallet);
    profile_client.verify_trustline(employee);
    
    let vault_id = env.register(VaultContract, ());
    let vault_client = VaultContractClient::new(env, &vault_id);
    vault_client.initialize(admin);
    
    let token_address = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let token_client = token::Client::new(env, &token_address);
    let token_admin = token::StellarAssetClient::new(env, &token_address);
    
    vault_client.whitelist_asset(admin, &token_address, &true);
    token_admin.mint(employer, &1_000_000);
    
    let lifecycle_id = env.register(LifecycleContract, ());
    let lifecycle_client = LifecycleContractClient::new(env, &lifecycle_id);
    lifecycle_client.initialize(admin, &vault_id, &profile_id, &token_address);
    
    (lifecycle_client, vault_id, profile_id, token_address, token_client)
}

#[test]
fn test_process_fixed_payment() {
    let (env, admin, employer, employee) = setup_env();
    let (lifecycle_client, vault_id, _, token_address, token_client) = 
        setup_contracts(&env, &admin, &employer, &employee);
    
    let metadata = vestroll_common::ContractMetadata {
        employer: employer.clone(),
        employee: employee.clone(),
        contract_type: ContractType::FixedRate,
        status: ContractStatus::Active,
        amount: 1000,
        asset: token_address.clone(),
    };

    let contract_id = lifecycle_client.create_contract(
        &employer, &employee, &ContractType::FixedRate, &1000, &token_address, &metadata, &None,
    );
    
   
    token_client.approve(&employer, &employer, &3000, &10000);
    let vault_client = VaultContractClient::new(&env, &vault_id);
    
    vault_client.deposit(&employer, &1000, &token_address);
    
 
    token_client.transfer(&employer, &vault_id, &2000); 

    env.mock_all_auths_allowing_non_root_auth();
    
    lifecycle_client.process_fixed_payment(&employer, &contract_id, &500);
    
    assert_eq!(token_client.balance(&employee), 500);
}

#[test]
fn test_milestone_flow() {
    let (env, admin, employer, employee) = setup_env();
    let (lifecycle_client, vault_id, _, token_address, token_client) = 
        setup_contracts(&env, &admin, &employer, &employee);
    
    let milestones = Vec::from_array(&env, [
        Milestone { id: 1, description: String::from_str(&env, "P1"), amount: 300, completed: false, completed_at: None },
    ]);
    
    let metadata = vestroll_common::ContractMetadata {
        employer: employer.clone(),
        employee: employee.clone(),
        contract_type: ContractType::Milestone,
        status: ContractStatus::Active,
        amount: 1000,
        asset: token_address.clone(),
    };

    let contract_id = lifecycle_client.create_contract(
        &employer, &employee, &ContractType::Milestone, &1000, &token_address, &metadata, &Some(milestones),
    );
    
    token_client.approve(&employer, &employer, &2000, &10000);
    let vault_client = VaultContractClient::new(&env, &vault_id);
    
    vault_client.deposit(&employer, &1000, &token_address);
    token_client.transfer(&employer, &vault_id, &1000); // Buffer for "available" liquidity

    env.mock_all_auths_allowing_non_root_auth();
    
    lifecycle_client.complete_milestone(&employer, &contract_id, &1);
    lifecycle_client.process_milestone_payment(&employer, &contract_id, &1);
    
    assert_eq!(token_client.balance(&employee), 300);
}
