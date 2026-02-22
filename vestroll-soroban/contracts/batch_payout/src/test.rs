#![cfg(test)]

use crate::{BatchPayoutContract, BatchPayoutContractClient};
use soroban_sdk::{testutils::Address as _, token, Address, Env, Vec};
use vestroll_common::{Payment, VaultError};

fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::Client<'a> {
    let contract_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = token::Client::new(env, &contract_id.address());
    token_client
}

#[test]
fn test_successful_batch_payout() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let worker1 = Address::generate(&env);
    let worker2 = Address::generate(&env);
    let worker3 = Address::generate(&env);

    let token = create_token_contract(&env, &token_admin);
    let token_admin_client = token::StellarAssetClient::new(&env, &token.address);

    let contract_id = env.register(BatchPayoutContract, ());
    let client = BatchPayoutContractClient::new(&env, &contract_id);

    // Mint some token directly to the contract (simulating vault funding)
    token_admin_client.mint(&contract_id, &1000);

    // Assert initial balances
    assert_eq!(token.balance(&worker1), 0);
    assert_eq!(token.balance(&worker2), 0);
    assert_eq!(token.balance(&worker3), 0);
    assert_eq!(token.balance(&contract_id), 1000);

    // Create a vector of payments
    let mut payments = Vec::new(&env);
    payments.push_back(Payment {
        recipient: worker1.clone(),
        amount: 100,
    });
    payments.push_back(Payment {
        recipient: worker2.clone(),
        amount: 250,
    });
    payments.push_back(Payment {
        recipient: worker3.clone(),
        amount: 400,
    });

    // Process batch
    client.process(&token.address, &payments);

    // Assert end balances
    assert_eq!(token.balance(&worker1), 100);
    assert_eq!(token.balance(&worker2), 250);
    assert_eq!(token.balance(&worker3), 400);
    assert_eq!(token.balance(&contract_id), 250); // 1000 - 750
}

#[test]
fn test_insufficient_contract_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let worker1 = Address::generate(&env);
    let worker2 = Address::generate(&env);

    let token = create_token_contract(&env, &token_admin);
    let token_admin_client = token::StellarAssetClient::new(&env, &token.address);

    let contract_id = env.register(BatchPayoutContract, ());
    let client = BatchPayoutContractClient::new(&env, &contract_id);

    // Mint only 100 tokens to the contract
    token_admin_client.mint(&contract_id, &100);

    // Try to payout 150
    let mut payments = Vec::new(&env);
    payments.push_back(Payment {
        recipient: worker1.clone(),
        amount: 75,
    });
    payments.push_back(Payment {
        recipient: worker2.clone(),
        amount: 75,
    });

    let result = client.try_process(&token.address, &payments);
    assert_eq!(result, Err(Ok(VaultError::InsufficientBalance)));
}
