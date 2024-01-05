use candid::{CandidType, Nat};
use thiserror::Error;

#[derive(Error, Debug, CandidType)]
pub enum ApolloError {
    #[error("ApolloInstanceError: {0}")]
    ApolloInstanceError(#[from] ApolloInstanceError),

}


#[derive(Error, Debug, CandidType)]
pub enum ApolloInstanceError {
    #[error("Failed to create: {0}")]
    FailedToCreate(String),
    #[error("Failed to install code: {0}")]
    FailedToInstallCode(String),
    #[error("Chain already exists: {0}")]
    ChainAlreadyExists(Nat),
}



