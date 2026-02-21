use soroban_sdk::{contracttype, Address, String};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Initialized,
    Profile(Address),
    WorkerWallet(Address),
    OrgWorkers(Address),
    WorkerCount(Address),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerWallet {
    pub worker_id: Address,
    pub wallet_address: String,
    pub trustline_verified: bool,
    pub last_verified: u64,
    pub is_active: bool,
}