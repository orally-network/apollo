use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct ApolloInstance {
    pub canister_id: Principal,
    pub chain_id: Nat,
    pub is_active: bool,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct AddApolloInstanceRequest {
    pub apollos_fee: Nat,
    pub chain_id: Nat,
    pub chain_rpc: String,
    pub apollo_coordinator: String,
    pub multicall_address: String,
    pub timer_frequency_sec: u64,
    pub block_gas_limit: Nat,
    pub min_balance: Nat,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct GetApolloInstancesFilter {
    pub chain_id: Option<Nat>,
    pub search: Option<String>,
}
