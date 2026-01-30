#![cfg(test)]
use soroban_sdk::{
    Address, Env, testutils::{Address as _}, token
};
use crate::{VaultContract, VaultContractClient};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> (token::Client<'a>, token::StellarAssetClient<'a>) {
     let contract_id = env.register_stellar_asset_contract_v2(admin.clone()).address();
    (
        token::Client::new(env, &contract_id),
        token::StellarAssetClient::new(env, &contract_id)
    )
}

fn create_test_env() -> (
    Env,
    VaultContractClient<'static>,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VaultContract, ());
    let client = VaultContractClient::new(&env, &contract_id);

    (env, client, contract_id)
}

#[test]
fn test_initilization() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);
}

#[test]
fn test_whitelist_asset() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let (token, _) = create_token_contract(&env, &token_admin);

    // Whitelist
    client.whitelist_asset(&admin, &token.address, &true);
    
    // Remove
    client.whitelist_asset(&admin, &token.address, &false);
}

#[test]
fn test_set_protocol_asset() {
     let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    
    let token_admin = Address::generate(&env);
    let (token, _) = create_token_contract(&env, &token_admin);
    
    client.set_protocol_asset(&admin, &token.address);
}

#[test]
fn test_deposit_success() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token_contract(&env, &token_admin);
    let from = Address::generate(&env);

    client.whitelist_asset(&admin, &token.address, &true);

    token_admin_client.mint(&from, &10000);
    token.approve(&from, &contract_id, &10000, &200);
    
    // Deposit
    client.deposit(&from, &500, &token.address);
    
    assert_eq!(token.balance(&contract_id), 500);
    assert_eq!(token.balance(&from), 9500);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")] // AssetNotWhitelisted
fn test_deposit_not_whitelisted() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token_contract(&env, &token_admin);
    let from = Address::generate(&env);
    
    
    token_admin_client.mint(&from, &1000);
    token.approve(&from, &contract_id, &1000, &200);

    client.deposit(&from, &500, &token.address);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")] // InvalidAmount
fn test_deposit_zero_amount() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);
     let token_admin = Address::generate(&env);
    let (token, _) = create_token_contract(&env, &token_admin);
    let from = Address::generate(&env);
    client.whitelist_asset(&admin, &token.address, &true);
    
    client.deposit(&from, &0, &token.address);
}


#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")] // ContractPaused
fn test_deposit_when_paused() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    
    let token_admin = Address::generate(&env);
    let (_token, _) = create_token_contract(&env, &token_admin); // Warning: mint not available here if destructure is wrong, but waiting panic anyway.
    let from = Address::generate(&env);
    
    // Fix: need client to mint/approve to actally test PAUSE logic inside deposit not failing before transfer. 
    // Wait, deposit contract paused check is BEFORE transfer. So we don't strictly need approval if it fails early.
    // BUT consistent setup is better. Let's create proper clients.
     let (token, token_admin_client) = create_token_contract(&env, &token_admin);

    client.whitelist_asset(&admin, &token.address, &true);
    client.set_pause(&admin, &true);

    token_admin_client.mint(&from, &1000);
    token.approve(&from, &contract_id, &1000, &200);

    client.deposit(&from, &100, &token.address);
}

#[test]
fn test_withdraw_success() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    
    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token_contract(&env, &token_admin);
    let to = Address::generate(&env);
    
    client.whitelist_asset(&admin, &token.address, &true);
    
    // Fund contract
    token_admin_client.mint(&contract_id, &1000);
    
    client.withdraw(&to, &200, &token.address);
    
    assert_eq!(token.balance(&to), 200);
    assert_eq!(token.balance(&contract_id), 800);
}