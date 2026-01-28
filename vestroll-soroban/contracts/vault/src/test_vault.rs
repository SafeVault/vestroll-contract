#![cfg(test)]
use soroban_sdk::{
    Address, Env, String, log, symbol_short, testutils::Address as _, token
};
use soroban_sdk::{IntoVal, Symbol, TryFromVal, Val};

use soroban_sdk::testutils::{Events};
use crate::{VaultContract, VaultContractClient};

fn create_test_env() -> (
    Env,
    VaultContractClient<'static>,
    Address,
) {
    let env = Env::default();
    let contract_id = env.register_contract(None, VaultContract);
    let client = VaultContractClient::new(&env, &contract_id);

    (env, client, contract_id)
}

#[test]
fn test_initilization() {
    let (env, client, _contract_id) = create_test_env();
    let _employee = Address::generate(&env);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize
    client.initialize(&admin);
}


#[test]
fn test_deposit_while_not_paused() {
    let (env, client, _contract_id) = create_test_env();
    let _employee = Address::generate(&env);

    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let asset = Address::generate(&env);
    let amount = 10000;
    env.mock_all_auths();

    // Initialize
    client.initialize(&admin);

    client.deposit(&from, &amount, &asset);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_deposit_when_paused() {
    let (env, client, _contract_id) = create_test_env();
    let _employee = Address::generate(&env);

    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let asset = Address::generate(&env);
    let amount = 10000;
    env.mock_all_auths();

    // Initialize
    client.initialize(&admin);

    client.set_pause(&admin, &true);

    client.deposit(&from, &amount, &asset);
}


#[test]
#[should_panic(expected = "HostError: Error(Contract, #2)")]
fn test_unauthorized_paused() {
    let (env, client, _contract_id) = create_test_env();
    let _employee = Address::generate(&env);

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
    let _employee = Address::generate(&env);

   let admin = Address::generate(&env);
    env.mock_all_auths();

    client.set_pause(&admin, &true);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_pause_when_paused() {
    let (env, client, _contract_id) = create_test_env();
    let _employee = Address::generate(&env);

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
    let _employee = Address::generate(&env);

    let admin = Address::generate(&env);
    let from = Address::generate(&env);
    let asset = Address::generate(&env);
    let amount = 10000;
    env.mock_all_auths();

    // Initialize
    client.initialize(&admin);

    client.set_pause(&admin, &true);
    let events = env.events().all();
    assert_eq!(events.len(), 1);
    log!(&env, "EVENTCAPTURED 1:", events);


    client.set_pause(&admin, &false);
    let events = env.events().all();
    assert_eq!(events.len(), 1);

    log!(&env, "EVENTCAPTURED 2:", events);
}