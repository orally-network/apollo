use std::cell::RefCell;

use candid::{CandidType, Nat};
use ic_stable_structures::{StableCell, StableBTreeMap};
use serde::{Serialize, Deserialize};

use crate::memory::{Cbor, VMemory};

use self::apollo_instance::ApolloInstance;

pub mod apollo_instance;
pub mod candid_types;


#[derive(Serialize, Deserialize, Default, CandidType, Clone)]
pub struct ApolloIntanceMetadata {
    pub tx_fee: Nat,
    pub key_name: String,
    pub chain_id: Nat,
}


#[derive(Serialize, Deserialize, Default, CandidType, Clone)]
pub struct Metadata {
    pub tx_fee: Nat,
    pub key_name: String,
    pub apollo_evm_contract: String, 
    // Apollo will check the ${apollo_evm_contract} contract for new requests every ${timer_frequency} seconds
    pub timer_frequency: Nat,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    
    #[serde(skip, default = "init_metadata")]
    pub metadata: StableCell<Cbor<Metadata>, VMemory>,

    #[serde(skip, default = "init_chains")]
    pub chains: StableBTreeMap<u32, Cbor<ApolloInstance>, VMemory>,
}

thread_local! {
    pub static STATE: RefCell<State> = RefCell::new(State::default());
}

fn init_chains() -> StableBTreeMap<u32, Cbor<ApolloInstance>, VMemory> {
    StableBTreeMap::init(crate::memory::get_stable_btree_memory())
}

fn init_metadata() -> StableCell<Cbor<Metadata>, VMemory> {
    let metadata = Cbor(Metadata::default());
    StableCell::init(crate::memory::get_metadata_memory(), metadata).unwrap()
}

impl Default for State {
    fn default() -> Self {
        Self {
            metadata: init_metadata(),
            chains: init_chains(),
        }
    }
}