#![cfg(test)]
use soroban_sdk::TryFromVal;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

use crate::{VaultContract, VaultContractClient};
use soroban_sdk::testutils::Events;
use vestroll_common::UNPAUSED;

fn create_test_env() -> (Env, VaultContractClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&env, &contract_id);

    (env, client, contract_id)
}

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>, Address) {
    let token_address = env.register_stellar_asset_contract(admin.clone());
    let client = token::Client::new(env, &token_address);
    let admin_client = token::StellarAssetClient::new(env, &token_address);
    (client, admin_client, token_address)
}

#[test]
fn test_initilization() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize
    client.initialize(&admin);
}

#[test]
fn test_deposit_while_not_paused() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Setup Token
    let (token_client, token_admin_client, token_address) = create_token_contract(&env, &admin);
    let amount = 10000;

    env.mock_all_auths();

    // Mint tokens to user
    token_admin_client.mint(&user, &amount);

    // Initialize
    client.initialize(&admin);

    client.deposit(&user, &amount, &token_address);

    // Verify stats
    let stats = client.get_treasury_stats(&token_address);
    assert_eq!(stats.total_deposits, amount);
    assert_eq!(stats.total_locked, amount);
    assert_eq!(stats.total_liquidity, 0);

    // Verify balance
    assert_eq!(token_client.balance(&contract_id), amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_deposit_when_paused() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Setup Token
    let (_token_client, _token_admin_client, token_address) = create_token_contract(&env, &admin);
    let amount = 10000;
    env.mock_all_auths();

    // Initialize
    client.initialize(&admin);

    client.set_pause(&admin, &true);

    client.deposit(&user, &amount, &token_address);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_unauthorized_paused() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let not_admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize
    client.initialize(&admin);

    client.set_pause(&not_admin, &true);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_admin_not_set() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.set_pause(&admin, &true);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_pause_when_paused() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize
    client.initialize(&admin);

    client.set_pause(&admin, &true);
    client.set_pause(&admin, &true);
}

#[test]
fn test_deposit_while_not_paused_event() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize
    client.initialize(&admin);

    client.set_pause(&admin, &true);
    let events = env.events().all();
    assert_eq!(events.len(), 1);

    client.set_pause(&admin, &false);
    let events = env.events().all();
    assert_eq!(events.len(), 1);

    // Verify it is UNPAUSED
    let event = events.last().unwrap();
    let topics = &event.1;
    assert_eq!(topics.len(), 2);
    let topic: soroban_sdk::Symbol =
        soroban_sdk::Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap();
    assert_eq!(topic, UNPAUSED);
}

#[test]
fn test_treasury_management() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Setup Token
    let (token_client, token_admin_client, token_address) = create_token_contract(&env, &admin);
    let deposit_amount = 1000;
    let surplus_amount = 500;

    env.mock_all_auths();

    // Initialize
    client.initialize(&admin);

    // 1. User deposits (Locked)
    token_admin_client.mint(&user, &deposit_amount);
    client.deposit(&user, &deposit_amount, &token_address);

    // 2. Add surplus (Direct transfer to vault, not via deposit)
    // Simulating fees or external funding
    token_admin_client.mint(&admin, &surplus_amount);
    token_client.transfer(&admin, &contract_id, &surplus_amount);

    // Verify Stats
    let stats = client.get_treasury_stats(&token_address);
    assert_eq!(stats.total_deposits, deposit_amount);
    assert_eq!(stats.total_locked, deposit_amount);
    // Liquidity = Balance (1500) - Locked (1000) = 500
    assert_eq!(stats.total_liquidity, surplus_amount);

    // 3. Admin attempts to withdraw locked funds (Should Fail)

    // 4. Admin withdraws available successfully
    let recipient = Address::generate(&env);
    client.withdraw_available(&recipient, &surplus_amount, &token_address);

    // Verify transfer
    assert_eq!(token_client.balance(&recipient), surplus_amount);
    assert_eq!(token_client.balance(&contract_id), deposit_amount);

    // Verify Stats
    let stats_after = client.get_treasury_stats(&token_address);
    assert_eq!(stats_after.total_liquidity, 0);
    assert_eq!(stats_after.total_locked, deposit_amount);
}

#[test]
#[should_panic(expected = "Insufficient unallocated funds")]
fn test_withdraw_available_fail_locked() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (_token_client, token_admin_client, token_address) = create_token_contract(&env, &admin);
    let amount = 1000;
    env.mock_all_auths();
    client.initialize(&admin);

    token_admin_client.mint(&user, &amount);
    client.deposit(&user, &amount, &token_address);

    // Try to withdraw locked funds
    let recipient = Address::generate(&env);
    client.withdraw_available(&recipient, &500, &token_address);
}
