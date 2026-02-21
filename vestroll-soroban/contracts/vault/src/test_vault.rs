#![cfg(test)]
use soroban_sdk::{testutils::Address as _, token, Address, Env};
use vestroll_common::PayoutEntry;

use crate::{VaultContract, VaultContractClient};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn create_test_env() -> (Env, VaultContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VaultContract, ());
    let client = VaultContractClient::new(&env, &contract_id);
    (env, client, contract_id)
}

/// Unified token creation helper to resolve v1 vs standard naming conflicts
fn create_token_contract<'a>(
    env: &'a Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>, Address) {
    let token_address = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let client = token::Client::new(env, &token_address);
    let admin_client = token::StellarAssetClient::new(env, &token_address);
    (client, admin_client, token_address)
}

fn setup_funded_vault(
    env: &Env,
    client: &VaultContractClient,
    contract_id: &Address,
    admin: &Address,
    amount: i128,
) -> (token::Client<'static>, Address) {
    let (token_client, token_admin_client, token_address) = create_token_contract(env, admin);

    client.initialize(admin);
    client.whitelist_asset(admin, &token_address, &true);

    token_admin_client.mint(admin, &amount);
    token_client.approve(admin, contract_id, &amount, &200);
    client.deposit(admin, &amount, &token_address);

    (token_client, token_address)
}

// ── Basic Configuration Tests ────────────────────────────────────────────────

#[test]
fn test_initialization() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);
    assert!(!client.is_paused());
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
    client.blacklist_asset(&admin, &token_address);
    assert!(!client.is_asset_whitelisted(&token_address));
}

#[test]
fn test_pause_toggle() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    client.set_pause(&admin, &true);
    assert!(client.is_paused());
    client.set_pause(&admin, &false);
    assert!(!client.is_paused());
}

// ── Treasury & Deposit Tests ────────────────────────────────────────────────

#[test]
fn test_deposit_success() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let (token, token_admin_client, token_address) = create_token_contract(&env, &admin);
    let from = Address::generate(&env);

    client.whitelist_asset(&admin, &token_address, &true);
    token_admin_client.mint(&from, &10000);
    token.approve(&from, &contract_id, &10000, &200);
    client.deposit(&from, &500, &token_address);

    assert_eq!(token.balance(&contract_id), 500);
    assert_eq!(token.balance(&from), 9500);
}

#[test]
fn test_treasury_management() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (token_client, token_admin_client, token_address) = create_token_contract(&env, &admin);
    let deposit_amount = 1000;
    let surplus_amount = 500;

    client.initialize(&admin);
    client.whitelist_asset(&admin, &token_address, &true);

    token_admin_client.mint(&user, &deposit_amount);
    token_client.approve(&user, &contract_id, &deposit_amount, &200);
    client.deposit(&user, &deposit_amount, &token_address);

    // Simulate "Unallocated" funds (liquidity) by transferring directly to contract
    token_admin_client.mint(&admin, &surplus_amount);
    token_client.transfer(&admin, &contract_id, &surplus_amount);

    let stats = client.get_treasury_stats(&token_address);
    assert_eq!(stats.total_deposits, deposit_amount);
    assert_eq!(stats.total_liquidity, surplus_amount);

    let recipient = Address::generate(&env);
    client.withdraw_available(&recipient, &surplus_amount, &token_address);

    assert_eq!(token_client.balance(&recipient), surplus_amount);
    assert_eq!(token_client.balance(&contract_id), deposit_amount);
}

// ── Error Condition Tests ────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_deposit_when_paused() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let amount = 1000;

    let (_, _, token_address) = create_token_contract(&env, &admin);
    client.initialize(&admin);
    client.whitelist_asset(&admin, &token_address, &true);
    
    client.set_pause(&admin, &true);
    client.deposit(&user, &amount, &token_address);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_admin_not_set() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    // Trying to set pause before initialization should trigger auth error #1
    client.set_pause(&admin, &true);
}

// ── Batch Payout Tests ───────────────────────────────────────────────────────

#[test]
fn test_execute_payouts_multiple_recipients() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);

    let (token, token_address) = setup_funded_vault(&env, &client, &contract_id, &admin, 3000);

    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient: r1.clone(), amount: 500,  asset: token_address.clone() },
        PayoutEntry { recipient: r2.clone(), amount: 1000, asset: token_address.clone() },
    ];

    let processed = client.execute_payouts(&contract_id, &list);
    assert_eq!(processed, 2);
    assert_eq!(token.balance(&r1), 500);
    assert_eq!(token.balance(&r2), 1000);
}

#[test]
fn test_execute_payouts_atomic_rollback() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let r1 = Address::generate(&env);

    let (token, token_address) = setup_funded_vault(&env, &client, &contract_id, &admin, 1000);

    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient: r1.clone(), amount: 400,  asset: token_address.clone() },
        PayoutEntry { recipient: Address::generate(&env), amount: 99999, asset: token_address.clone() }, 
    ];

    // try_execute_payouts should fail and roll back the 400 payment to r1
    let result = client.try_execute_payouts(&contract_id, &list);
    assert!(result.is_err());

    assert_eq!(token.balance(&r1), 0);
    assert_eq!(token.balance(&contract_id), 1000);
}