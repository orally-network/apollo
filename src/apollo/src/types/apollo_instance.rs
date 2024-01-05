use candid::{Principal, CandidType};
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct ApolloInstance {
    pub canister_id: Principal,
    pub is_active: bool,
}




