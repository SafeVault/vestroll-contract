use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProfileType {
    Organization,
    Worker,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    pub id: Address,
    pub name: String,
    pub profile_type: ProfileType,
    pub created_at: u64,
    pub is_active: bool,
}