#![cfg(test)]

use super::{ProfileContract, ProfileContractClient};
use crate::types::ProfileType;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, String, symbol_short};

fn setup_test() -> (Env, ProfileContractClient<'static>, Address) {
    let env = Env::default();
    
    env.ledger().with_mut(|li| {
        li.timestamp = 12345;
        li.sequence_number = 1000;
    });

    env.mock_all_auths();

    let admin = Address::generate(&env);
    let contract_id = env.register(ProfileContract, ());
    let client = ProfileContractClient::new(&env, &contract_id);
    
    client.initialize(&admin);
    (env, client, admin)
}

fn create_test_worker(env: &Env, client: &ProfileContractClient, name: &str) -> Address {
    let worker = Address::generate(env);
    client.create_profile(&worker, &String::from_str(env, name), &false);
    worker
}

fn create_test_org(env: &Env, client: &ProfileContractClient, name: &str) -> Address {
    let org = Address::generate(env);
    client.create_profile(&org, &String::from_str(env, name), &true);
    org
}

fn valid_stellar_address(env: &Env) -> String {
    String::from_str(env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF")
}

#[test]
fn test_initialize() {
    let (_env, client, admin) = setup_test();
    assert!(client.is_initialized());
    assert_eq!(client.get_admin(), admin);
}

#[test]
#[should_panic]
fn test_initialize_twice() {
    let (_env, client, admin) = setup_test();
    client.initialize(&admin);
}

#[test]
fn test_create_worker_profile() {
    let (env, client, _admin) = setup_test();
    let user = Address::generate(&env);
    let name = String::from_str(&env, "John Doe");
    let profile = client.create_profile(&user, &name, &false);

    assert_eq!(profile.id, user);
    assert_eq!(profile.name, name);
    assert_eq!(profile.profile_type, ProfileType::Worker);
}

#[test]
fn test_get_profile() {
    let (env, client, _admin) = setup_test();
    let user = Address::generate(&env);
    let name = String::from_str(&env, "Jane Doe");
    client.create_profile(&user, &name, &false);

    let profile = client.get_profile(&user);
    assert_eq!(profile.name, name);
}

#[test]
fn test_deactivate_profile() {
    let (env, client, _admin) = setup_test();
    let user = Address::generate(&env);
    client.create_profile(&user, &String::from_str(&env, "User"), &false);

    client.deactivate_profile(&user);
    let profile = client.get_profile(&user);
    assert!(!profile.is_active);
}

#[test]
fn test_register_worker_wallet() {
    let (env, client, _admin) = setup_test();
    let worker = create_test_worker(&env, &client, "Wallet User");
    let wallet = valid_stellar_address(&env);

    let result = client.register_worker_wallet(&worker, &wallet);
    assert_eq!(result.wallet_address, wallet);
}



#[test]
fn test_verify_trustline() {
    let (env, client, _admin) = setup_test();
    let worker = create_test_worker(&env, &client, "Trust User");
    client.register_worker_wallet(&worker, &valid_stellar_address(&env));
    
    assert!(client.verify_trustline(&worker));
    assert!(client.get_worker_wallet(&worker).trustline_verified);
}

#[test]
fn test_get_profile_stats_worker() {
    let (env, client, _admin) = setup_test();
    let worker = create_test_worker(&env, &client, "Stats User");
    client.register_worker_wallet(&worker, &valid_stellar_address(&env));
    client.verify_trustline(&worker);
    
    let stats = client.get_profile_stats(&worker);
    assert!(stats.get(symbol_short!("created")).unwrap() > 0);
    assert_eq!(stats.get(symbol_short!("trust")).unwrap(), 1);
}

#[test]
fn test_get_profile_stats_organization() {
    let (env, client, _admin) = setup_test();
    let org = create_test_org(&env, &client, "Stats Org");
    let worker = create_test_worker(&env, &client, "Worker 1");
    
    client.add_worker_to_organization(&org, &worker);
    let stats = client.get_profile_stats(&org);
    
    assert!(stats.get(symbol_short!("created")).unwrap() > 0);
    assert_eq!(stats.get(symbol_short!("workers")).unwrap(), 1);
}

#[test]
#[should_panic]
fn test_operations_without_initialization() {
    let env = Env::default();
    let contract_id = env.register(ProfileContract, ());
    let client = ProfileContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    client.create_profile(&user, &String::from_str(&env, "Fail"), &false);
}

