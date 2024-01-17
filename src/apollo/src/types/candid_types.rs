use candid::{CandidType, Nat};
use serde::{Serialize, Deserialize};





#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct AddChainRequest {
    pub chain_id: Nat,
    pub chain_rpc: String,
    // Frequency in seconds to check for new blocks
    pub timer_frequency: u64,
}