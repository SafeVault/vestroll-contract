use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    ProfileNotFound = 3,
    Unauthorized = 4,
    WalletAlreadyRegistered = 5,
    InvalidWalletAddress = 6,
    TrustlineCheckFailed = 7,
    MissingTrustline = 8,
    NotAWorker = 9,
    NotAnOrganization = 10,
}