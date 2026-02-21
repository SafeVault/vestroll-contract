#![cfg(test)]
use soroban_sdk::{testutils::Address as _, token, Address, Env};
use vestroll_common::PayoutEntry;

use crate::{VaultContract, VaultContractClient};

fn create_test_env() -> (Env, VaultContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(VaultContract, ());
    let client = VaultContractClient::new(&env, &contract_id);
    (env, client, contract_id)
}

fn create_token_contract_v1<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>, Address) {
    let token_address = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let client = token::Client::new(env, &token_address);
    let admin_client = token::StellarAssetClient::new(env, &token_address);
    (client, admin_client, token_address)
}

// ── helper: deposit funds into vault and return token client + address ────────
fn setup_funded_vault(
    env: &Env,
    client: &VaultContractClient,
    contract_id: &Address,
    admin: &Address,
    amount: i128,
) -> (token::Client<'static>, Address) {
    let (token_client, token_admin_client, token_address) =
        create_token_contract_v1(env, admin);

    client.initialize(admin);
    client.whitelist_asset(admin, &token_address, &true);

    token_admin_client.mint(admin, &amount);
    token_client.approve(admin, contract_id, &amount, &200);
    client.deposit(admin, &amount, &token_address);

    (token_client, token_address)
}

// ── existing tests (unchanged) ────────────────────────────────────────────────

#[test]
fn test_initilization() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);
}

#[test]
fn test_deposit_while_not_paused() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (token_client, token_admin_client, token_address) = create_token_contract_v1(&env, &admin);
    let amount = 10000;

    env.mock_all_auths();

    token_admin_client.mint(&user, &amount);
    client.initialize(&admin);
    client.whitelist_asset(&admin, &token_address, &true);
    token_client.approve(&user, &contract_id, &amount, &200);
    client.deposit(&user, &amount, &token_address);

    let stats = client.get_treasury_stats(&token_address);
    assert_eq!(stats.total_deposits, amount);
    assert_eq!(stats.total_locked, amount);
    assert_eq!(stats.total_liquidity, 0);
    assert_eq!(token_client.balance(&contract_id), amount);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_deposit_when_paused() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (_token_client, _token_admin_client, token_address) = create_token_contract_v1(&env, &admin);
    let amount = 10000;
    env.mock_all_auths();

    client.initialize(&admin);
    client.whitelist_asset(&admin, &token_address, &true);
    client.set_pause(&admin, &true);
    client.deposit(&user, &amount, &token_address);
}

#[test]
fn test_whitelist_asset() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let (token, _, _) = create_token_contract_v1(&env, &token_admin);

    client.whitelist_asset(&admin, &token.address, &true);
    client.whitelist_asset(&admin, &token.address, &false);
}

#[test]
fn test_set_protocol_asset() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let (token, _, _) = create_token_contract_v1(&env, &token_admin);

    client.set_protocol_asset(&admin, &token.address);
}

#[test]
fn test_deposit_success() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let (token, token_admin_client, _) = create_token_contract_v1(&env, &token_admin);
    let from = Address::generate(&env);

    client.whitelist_asset(&admin, &token.address, &true);
    token_admin_client.mint(&from, &10000);
    token.approve(&from, &contract_id, &10000, &200);
    client.deposit(&from, &500, &token.address);

    assert_eq!(token.balance(&contract_id), 500);
    assert_eq!(token.balance(&from), 9500);
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
#[should_panic(expected = "HostError: Error(Contract, #4)")]
fn test_deposit_not_whitelisted() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let (token, token_admin_client, _) = create_token_contract_v1(&env, &token_admin);
    let from = Address::generate(&env);

    token_admin_client.mint(&from, &1000);
    client.deposit(&from, &500, &token.address);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_deposit_zero_amount() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    env.mock_all_auths();
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let (token, _, _) = create_token_contract_v1(&env, &token_admin);
    let from = Address::generate(&env);

    client.whitelist_asset(&admin, &token.address, &true);
    client.deposit(&from, &0, &token.address);
}

#[test]
fn test_treasury_management() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (token_client, token_admin_client, token_address) = create_token_contract_v1(&env, &admin);
    let deposit_amount = 1000;
    let surplus_amount = 500;

    env.mock_all_auths();

    client.initialize(&admin);
    client.whitelist_asset(&admin, &token_address, &true);

    token_admin_client.mint(&user, &deposit_amount);
    token_client.approve(&user, &contract_id, &deposit_amount, &200);
    client.deposit(&user, &deposit_amount, &token_address);

    token_admin_client.mint(&admin, &surplus_amount);
    token_client.transfer(&admin, &contract_id, &surplus_amount);

    let stats = client.get_treasury_stats(&token_address);
    assert_eq!(stats.total_deposits, deposit_amount);
    assert_eq!(stats.total_locked, deposit_amount);
    assert_eq!(stats.total_liquidity, surplus_amount);

    let recipient = Address::generate(&env);
    client.withdraw_available(&recipient, &surplus_amount, &token_address);

    assert_eq!(token_client.balance(&recipient), surplus_amount);
    assert_eq!(token_client.balance(&contract_id), deposit_amount);

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

    let (token_client, token_admin_client, token_address) = create_token_contract_v1(&env, &admin);
    let amount = 1000;
    env.mock_all_auths();
    client.initialize(&admin);
    client.whitelist_asset(&admin, &token_address, &true);

    token_admin_client.mint(&user, &amount);
    token_client.approve(&user, &_contract_id, &amount, &200);
    client.deposit(&user, &amount, &token_address);

    let recipient = Address::generate(&env);
    client.withdraw_available(&recipient, &500, &token_address);
}

#[test]
fn test_withdraw_success() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let (token, token_admin_client, _) = create_token_contract_v1(&env, &token_admin);
    let to = Address::generate(&env);

    client.whitelist_asset(&admin, &token.address, &true);
    token_admin_client.mint(&admin, &1000);
    token.approve(&admin, &contract_id, &1000, &200);
    client.deposit(&admin, &1000, &token.address);
    client.withdraw(&to, &200, &token.address);

    assert_eq!(token.balance(&to), 200);
    assert_eq!(token.balance(&contract_id), 800);
}

#[test]
fn test_blacklist_asset() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let token_admin = Address::generate(&env);
    let (token, _, _) = create_token_contract_v1(&env, &token_admin);

    client.whitelist_asset(&admin, &token.address, &true);
    assert!(client.is_asset_whitelisted(&token.address));

    client.blacklist_asset(&admin, &token.address);
    assert!(!client.is_asset_whitelisted(&token.address));
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #11)")]
fn test_withdraw_insufficient_balance() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let (token, token_admin_client, _) = create_token_contract_v1(&env, &admin);
    let to = Address::generate(&env);

    client.whitelist_asset(&admin, &token.address, &true);
    token_admin_client.mint(&_contract_id, &100);
    client.withdraw(&to, &200, &token.address);
}

// ── NEW: batch payout tests ───────────────────────────────────────────────────

#[test]
fn test_execute_payouts_single_entry() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token, token_address) =
        setup_funded_vault(&env, &client, &contract_id, &admin, 1000);

    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient: recipient.clone(), amount: 300, asset: token_address.clone() },
    ];

    let processed = client.execute_payouts(&contract_id, &list);
    assert_eq!(processed, 1);
    assert_eq!(token.balance(&recipient), 300);
    assert_eq!(token.balance(&contract_id), 700);

    let stats = client.get_treasury_stats(&token_address);
    assert_eq!(stats.total_locked, 700);
}

#[test]
fn test_execute_payouts_multiple_recipients() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let r3 = Address::generate(&env);

    let (token, token_address) =
        setup_funded_vault(&env, &client, &contract_id, &admin, 3000);

    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient: r1.clone(), amount: 500,  asset: token_address.clone() },
        PayoutEntry { recipient: r2.clone(), amount: 1000, asset: token_address.clone() },
        PayoutEntry { recipient: r3.clone(), amount: 750,  asset: token_address.clone() },
    ];

    let processed = client.execute_payouts(&contract_id, &list);
    assert_eq!(processed, 3);
    assert_eq!(token.balance(&r1), 500);
    assert_eq!(token.balance(&r2), 1000);
    assert_eq!(token.balance(&r3), 750);
    assert_eq!(token.balance(&contract_id), 750);
}

#[test]
fn test_execute_payouts_atomic_rollback() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);

    let (token, token_address) =
        setup_funded_vault(&env, &client, &contract_id, &admin, 1000);

    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient: r1.clone(), amount: 400,  asset: token_address.clone() },
        PayoutEntry { recipient: r2.clone(), amount: 9999, asset: token_address.clone() }, // too large
    ];

    let result = client.try_execute_payouts(&contract_id, &list);
    assert!(result.is_err());

    // Full rollback — vault unchanged, r1 got nothing
    assert_eq!(token.balance(&contract_id), 1000);
    assert_eq!(token.balance(&r1), 0);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #12)")]
fn test_execute_payouts_empty_list() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    setup_funded_vault(&env, &client, &contract_id, &admin, 500);

    let empty: soroban_sdk::Vec<PayoutEntry> = soroban_sdk::vec![&env];
    client.execute_payouts(&contract_id, &empty);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_execute_payouts_when_paused() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (_, token_address) =
        setup_funded_vault(&env, &client, &contract_id, &admin, 1000);

    client.set_pause(&admin, &true);

    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient, amount: 100, asset: token_address },
    ];
    client.execute_payouts(&contract_id, &list);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_execute_payouts_wrong_vault_address() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (_, token_address) =
        setup_funded_vault(&env, &client, &contract_id, &admin, 500);

    let wrong_vault = Address::generate(&env);
    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient, amount: 100, asset: token_address },
    ];
    client.execute_payouts(&wrong_vault, &list);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #5)")]
fn test_execute_payouts_zero_amount() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (_, token_address) =
        setup_funded_vault(&env, &client, &contract_id, &admin, 500);

    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient, amount: 0, asset: token_address },
    ];
    client.execute_payouts(&contract_id, &list);
}