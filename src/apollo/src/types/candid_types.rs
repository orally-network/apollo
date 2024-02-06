use candid::{CandidType, Nat};
use serde::{Deserialize, Serialize};

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
