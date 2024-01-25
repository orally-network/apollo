use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct ApolloInstance {
    pub canister_id: Principal,
    pub is_active: bool,
}
