use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct ApolloInstance {
    pub canister_id: Principal,
    pub chain_id: Nat,
    pub is_active: bool,
}
