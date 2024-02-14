/// These types are created in order to generate proper name for the struct
/// in the generated candid file.
use apollo_utils::{apollo_instance::ApolloInstanceMetadata, errors::ApolloError};
use candid::{CandidType, Nat};

use super::apollo_instance::ApolloInstance;

#[derive(Debug, CandidType)]
pub enum StringResult {
    Ok(String),
    Err(ApolloError),
}

#[derive(Debug, CandidType)]
pub enum ApolloInstanceMetadataResult {
    Ok(ApolloInstanceMetadata),
    Err(ApolloError),
}

#[derive(Debug, CandidType)]
pub enum NatResult {
    Ok(Nat),
    Err(ApolloError),
}

#[derive(Debug, CandidType)]
pub struct GetApolloInstanceResult {
    pub chain_id: u32,
    pub apollo_instance: ApolloInstance,
}
