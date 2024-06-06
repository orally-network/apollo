use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType, Clone, Debug)]
pub struct ApolloInstance {
    pub canister_id: Principal,
    pub chain_id: Nat,
    #[serde(default)]
    pub apollo_coordinator: String,
    #[serde(default)]
    pub apollo_main_address: String,
    pub is_active: bool,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct AddApolloInstanceRequest {
    pub apollos_fee: Nat,
    pub chain_id: Nat,
    pub chain_rpc: String,
    pub apollo_coordinator: String,
    pub multicall_address: String,
    pub evm_rpc_canister: String,
    pub timer_frequency_sec: u64,
    pub block_gas_limit: Nat,
    pub min_balance: Nat,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct GetApolloInstancesFilter {
    pub chain_id: Option<Nat>,
    pub search: Option<String>,
}
