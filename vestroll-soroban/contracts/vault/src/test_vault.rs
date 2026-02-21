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
    env: &Env,
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

    // Deposit
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
    let user = Address::generate(&env);

    client.initialize(&admin, &token_address);

    let amount = 1000;
    token_admin_client.mint(&contract_id, &amount);

    // Admin withdraws
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

    // Non-admin tries to withdraw - should fail since mock_all_auths is on but it requires admin.require_auth()
    // and the implementation checks if the address calling is admin (implicit in require_auth and storage check)
    // Actually, require_auth() works with the contract implementation.
    // In our implementation:
    // let admin = env.storage().instance().get(&DataKey::Admin).unwrap();
    // admin.require_auth();

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
    let (env, client, contract_id) = create_test_env();
    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let (_, token_admin_client, token_address) = create_token_contract(&env, &token_admin);
    let user = Address::generate(&env);

    client.initialize(&admin, &token_address);
    client.set_pause(&admin, &true);

    token_admin_client.mint(&user, &1000);

    // Deposit should fail when paused
    let result = client.try_deposit(&user, &500);
    assert!(result.is_err());

    // Resume
    client.set_pause(&admin, &false);
    client.deposit(&user, &500);
    assert_eq!(client.try_deposit(&user, &500).is_ok(), true);
}
