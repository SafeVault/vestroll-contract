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

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::Client<'a>, token::StellarAssetClient<'a>, Address) {
    let token_address = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let client = token::Client::new(env, &token_address);
    let admin_client = token::StellarAssetClient::new(env, &token_address);
    (client, admin_client, token_address)
}

// Helper: deposit funds into vault and return token client + address
fn setup_funded_vault(
    env: &Env,
    client: &VaultContractClient,
    contract_id: &Address,
    admin: &Address,
    amount: i128,
) -> (token::Client<'static>, Address) {
    let (token_client, token_admin_client, token_address) = create_token_contract(env, admin);

    client.initialize(admin, &token_address);

    token_admin_client.mint(admin, &amount);
    token_client.approve(admin, contract_id, &amount, &200);
    client.deposit(admin, &amount);

    (token_client, token_address)
}

#[test]
fn test_initialization() {
    let (env, client, _) = create_test_env();
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (_, _, token_address) = create_token_contract(&env, &token_admin);

    client.initialize(&admin, &token_address);

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_token(), token_address);
}

#[test]
fn test_deposit() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_client, token_admin_client, token_address) = create_token_contract(&env, &token_admin);
    let user = Address::generate(&env);

    client.initialize(&admin, &token_address);

    let amount = 1000;
    token_admin_client.mint(&user, &amount);

    client.deposit(&user, &amount);

    assert_eq!(token_client.balance(&contract_id), amount);
    assert_eq!(token_client.balance(&user), 0);
}

#[test]
fn test_withdraw_admin_only() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_client, token_admin_client, token_address) = create_token_contract(&env, &token_admin);

    client.initialize(&admin, &token_address);

    let amount = 1000;
    token_admin_client.mint(&contract_id, &amount);

    let recipient = Address::generate(&env);
    client.withdraw(&recipient, &500);

    assert_eq!(token_client.balance(&recipient), 500);
    assert_eq!(token_client.balance(&contract_id), 500);
}

#[test]
#[should_panic]
fn test_withdraw_non_admin_fails() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (_, token_admin_client, token_address) = create_token_contract(&env, &token_admin);

    client.initialize(&admin, &token_address);
    token_admin_client.mint(&contract_id, &1000);

    env.as_contract(&non_admin, || {
         client.withdraw(&non_admin, &500);
    });
}

#[test]
fn test_transfer_to_contract() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (token_client, token_admin_client, token_address) = create_token_contract(&env, &token_admin);
    let other_contract = Address::generate(&env);

    client.initialize(&admin, &token_address);
    token_admin_client.mint(&contract_id, &1000);

    client.transfer_to_contract(&other_contract, &300);

    assert_eq!(token_client.balance(&other_contract), 300);
    assert_eq!(token_client.balance(&contract_id), 700);
}

#[test]
fn test_pause_mechanics() {
    let (env, client, _contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (_, token_admin_client, token_address) = create_token_contract(&env, &token_admin);
    let user = Address::generate(&env);

    client.initialize(&admin, &token_address);
    client.set_pause(&admin, &true);

    token_admin_client.mint(&user, &1000);

    let result = client.try_deposit(&user, &500);
    assert!(result.is_err());

    client.set_pause(&admin, &false);
    client.deposit(&user, &500);
    assert_eq!(client.try_deposit(&user, &500).is_ok(), true);
}

#[test]
fn test_execute_payouts_single_entry() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (token, token_address) = setup_funded_vault(&env, &client, &contract_id, &admin, 1000);

    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient: recipient.clone(), amount: 300, asset: token_address.clone() },
    ];

    let processed = client.execute_payouts(&contract_id, &list);
    assert_eq!(processed, 1);
    assert_eq!(token.balance(&recipient), 300);
    assert_eq!(token.balance(&contract_id), 700);
}

#[test]
fn test_execute_payouts_multiple_recipients() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let r1 = Address::generate(&env);
    let r2 = Address::generate(&env);
    let r3 = Address::generate(&env);

    let (token, token_address) = setup_funded_vault(&env, &client, &contract_id, &admin, 3000);

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

    let (token, token_address) = setup_funded_vault(&env, &client, &contract_id, &admin, 1000);

    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient: r1.clone(), amount: 400,  asset: token_address.clone() },
        PayoutEntry { recipient: r2.clone(), amount: 9999, asset: token_address.clone() }, // too large
    ];

    let result = client.try_execute_payouts(&contract_id, &list);
    assert!(result.is_err());

    // Full rollback
    assert_eq!(token.balance(&contract_id), 1000);
    assert_eq!(token.balance(&r1), 0);
}

#[test]
#[should_panic]
fn test_execute_payouts_empty_list() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    setup_funded_vault(&env, &client, &contract_id, &admin, 500);

    let empty: soroban_sdk::Vec<PayoutEntry> = soroban_sdk::vec![&env];
    client.execute_payouts(&contract_id, &empty);
}

#[test]
#[should_panic]
fn test_execute_payouts_when_paused() {
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);

    let (_, token_address) = setup_funded_vault(&env, &client, &contract_id, &admin, 1000);

    client.set_pause(&admin, &true);

    let list = soroban_sdk::vec![
        &env,
        PayoutEntry { recipient, amount: 100, asset: token_address },
    ];
    client.execute_payouts(&contract_id, &list);
}
