use std::cell::RefCell;

use candid::{CandidType, Nat};
use ic_stable_structures::{StableCell};
use serde::{Serialize, Deserialize};

use crate::memory::{Cbor, VMemory};



#[derive(Serialize, Deserialize, Default, CandidType, Clone)]
pub struct Metadata {
    pub tx_fee: Nat,
    pub key_name: String,
    pub chain_id: Nat,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(skip, default = "init_metadata")]
    pub metadata: StableCell<Cbor<Metadata>, VMemory>,
}

thread_local! {
    pub static STATE: RefCell<State> = RefCell::new(State::default());
}

fn init_metadata() -> StableCell<Cbor<Metadata>, VMemory> {
    let metadata = Cbor(Metadata::default());
    StableCell::init(crate::memory::get_metadata_memory(), metadata).unwrap()
}

impl Default for State {
    fn default() -> Self {
        Self {
            metadata: init_metadata(),
        }
    }
}