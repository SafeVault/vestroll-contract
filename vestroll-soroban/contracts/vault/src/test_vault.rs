#![cfg(test)]
use soroban_sdk::{testutils::Address as _, token, Address, Env};

use crate::{VaultContract, VaultContractClient};

fn create_test_env() -> (Env, VaultContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VaultContract, ());
    let client = VaultContractClient::new(&env, &contract_id);
    (env, client, contract_id)
}

fn create_token_contract<'a>(
    env: &'a Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>, Address) {
    let token_address = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let client = token::Client::new(env, &token_address);
    let admin_client = token::StellarAssetClient::new(env, &token_address);
    (client, admin_client, token_address)
}

#[test]
fn test_initialization() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    assert!(!client.is_paused());
}


#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_deposit_when_paused() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let amount = 10000;

    client.initialize(&admin);

    let (token_client, token_admin_client, token_address) = create_token_contract(&env, &admin);
    client.whitelist_asset(&admin, &token_address, &true);

    token_admin_client.mint(&user, &amount);
    token_client.approve(&user, &contract_id, &amount, &200);

    client.set_pause(&admin, &true);
    client.deposit(&user, &amount, &token_address);
}

#[test]
fn test_whitelist_asset() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let (_, _, token_address) = create_token_contract(&env, &admin);

    client.whitelist_asset(&admin, &token_address, &true);
    assert!(client.is_asset_whitelisted(&token_address));

    client.whitelist_asset(&admin, &token_address, &false);
    assert!(!client.is_asset_whitelisted(&token_address));
}

#[test]
fn test_blacklist_asset() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let (_, _, token_address) = create_token_contract(&env, &admin);

    client.whitelist_asset(&admin, &token_address, &true);
    assert!(client.is_asset_whitelisted(&token_address));

    client.blacklist_asset(&admin, &token_address);
    assert!(!client.is_asset_whitelisted(&token_address));
}

#[test]
fn test_set_protocol_asset() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let (_, _, token_address) = create_token_contract(&env, &admin);
    client.set_protocol_asset(&admin, &token_address);
}


#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_admin_not_set() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    client.set_pause(&admin, &true);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_deposit_not_whitelisted() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let amount = 1000;

    client.initialize(&admin);

    let (token_client, token_admin_client, token_address) = create_token_contract(&env, &admin);

    token_admin_client.mint(&user, &amount);
    token_client.approve(&user, &contract_id, &amount, &200);
    client.deposit(&user, &amount, &token_address);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_deposit_zero_amount() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);

    let (token_client, token_admin_client, token_address) = create_token_contract(&env, &admin);
    client.whitelist_asset(&admin, &token_address, &true);

    token_admin_client.mint(&user, &1000);
    token_client.approve(&user, &contract_id, &1000, &200);
    client.deposit(&user, &0, &token_address);
}


#[test]
fn test_pause_toggle() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    assert!(!client.is_paused());
    client.set_pause(&admin, &true);
    assert!(client.is_paused());
    client.set_pause(&admin, &false);
    assert!(!client.is_paused());
}
