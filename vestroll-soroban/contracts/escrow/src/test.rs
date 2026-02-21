use crate::{EscrowContract, EscrowContractClient};
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token, Address, Env, IntoVal,
};
use vestroll_common::VaultError;

fn create_token_contract<'a>(env: &Env, admin: &Address) -> token::Client<'a> {
    let contract_id = env.register_stellar_asset_contract_v2(admin.clone());
    let token_client = token::Client::new(env, &contract_id.address());
    token_client
}

#[test]
fn test_successful_funding_and_release() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let funder = Address::generate(&env);
    let recipient = Address::generate(&env);

    let token = create_token_contract(&env, &token_admin);
    let token_admin_client = token::StellarAssetClient::new(&env, &token.address);
    token_admin_client.mint(&funder, &1000);

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    client.initialize(&admin, &token.address);

    // Initial assertions
    assert_eq!(client.total_funded(), 0);
    assert_eq!(client.released_amount(), 0);

    // Fund
    token_admin_client.mock_auths(&[MockAuth {
        address: &funder,
        invoke: &MockAuthInvoke {
            contract: &token.address,
            fn_name: "approve",
            args: (&funder, &contract_id, &500i128, &200u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    token.approve(&funder, &contract_id, &500, &200);

    client.fund(&funder, &500);

    assert_eq!(client.total_funded(), 500);
    assert_eq!(token.balance(&funder), 500);
    assert_eq!(token.balance(&contract_id), 500);

    // Release Milestone
    client.release_milestone(&1, &recipient, &200);

    assert_eq!(client.released_amount(), 200);
    assert_eq!(token.balance(&recipient), 200);
    assert_eq!(token.balance(&contract_id), 300);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_unauthorized_release() {
    let env = Env::default();

    let admin = Address::generate(&env);
    let bad_actor = Address::generate(&env);
    let recipient = Address::generate(&env);
    let funder = Address::generate(&env);
    let token_admin = Address::generate(&env);

    let token = create_token_contract(&env, &token_admin);
    let token_admin_client = token::StellarAssetClient::new(&env, &token.address);
    token_admin_client
        .mock_auths(&[MockAuth {
            address: &token_admin,
            invoke: &MockAuthInvoke {
                contract: &token.address,
                fn_name: "mint",
                args: (&funder, &1000i128).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .mint(&funder, &1000);

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    // Initialize with mock auth explicitly for admin so we can bypass the initial auth
    client
        .mock_auths(&[MockAuth {
            address: &admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "initialize",
                args: (&admin, &token.address).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .initialize(&admin, &token.address);

    // Fund
    token
        .mock_auths(&[MockAuth {
            address: &funder,
            invoke: &MockAuthInvoke {
                contract: &token.address,
                fn_name: "approve",
                args: (&funder, &contract_id, &500i128, &200u32).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .approve(&funder, &contract_id, &500, &200);

    client
        .mock_auths(&[MockAuth {
            address: &funder,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "fund",
                args: (&funder, &500i128).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .fund(&funder, &500);

    // Bad Actor tries to release milestone, expecting panic from `require_auth`
    client
        .mock_auths(&[MockAuth {
            address: &bad_actor,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "release_milestone",
                args: (&1u32, &recipient, &200i128).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .release_milestone(&1, &recipient, &200);
}

#[test]
fn test_over_release() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let recipient = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let funder = Address::generate(&env);

    let token = create_token_contract(&env, &token_admin);
    let token_admin_client = token::StellarAssetClient::new(&env, &token.address);
    token_admin_client.mint(&funder, &1000);

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    client.initialize(&admin, &token.address);

    // Fund 500
    token.approve(&funder, &contract_id, &500, &200);
    client.fund(&funder, &500);

    // Try to release 600 which is more than contract balance (500)
    let res = client.try_release_milestone(&1, &recipient, &600);
    assert_eq!(res, Err(Ok(VaultError::InsufficientBalance)));
}
