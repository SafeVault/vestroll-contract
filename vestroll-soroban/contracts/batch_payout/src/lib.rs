#![no_std]
#[cfg(test)]
mod test;
use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};
use vestroll_common::{Payment, VaultError};

#[contract]
pub struct BatchPayoutContract;

#[contractimpl]
impl BatchPayoutContract {
    /// Iterates through a list of payments and transfers the specified asset to each recipient.
    /// This contract must have the required asset balance before invocation.
    pub fn process(env: Env, asset: Address, payments: Vec<Payment>) -> Result<(), VaultError> {
        let client = token::Client::new(&env, &asset);

        let mut total_required: i128 = 0;
        for payment in payments.iter() {
            total_required += payment.amount;
        }

        let balance = client.balance(&env.current_contract_address());
        if total_required > balance {
            return Err(VaultError::InsufficientBalance);
        }

        for payment in payments.iter() {
            if payment.amount > 0 {
                client.transfer(
                    &env.current_contract_address(),
                    &payment.recipient,
                    &payment.amount,
                );
            }
        }

        Ok(())
    }
}
