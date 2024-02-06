use std::cell::RefCell;

use apollo_utils::memory::Cbor;
use candid::{CandidType, Nat};
use ic_stable_structures::{StableBTreeMap, StableCell};
use serde::{Deserialize, Serialize};

use crate::memory::VMemory;

use self::apollo_instance::ApolloInstance;

pub mod apollo_instance;
pub mod candid_types;

#[derive(Serialize, Deserialize, Default, CandidType, Clone)]
pub struct ApolloIntanceMetadata {
    pub tx_fee: Nat,
    pub key_name: String,
    pub chain_id: Nat,
}

#[derive(Serialize, Deserialize, Debug, Default, CandidType, Clone)]
pub struct Metadata {
    pub key_name: String,
    pub sybil_canister_address: String,
}

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct UpdateMetadata {
    sybil_canister_address: Option<String>,
}

impl Metadata {
    pub fn update(&mut self, update: UpdateMetadata) {
        if let Some(sybil_canister_address) = update.sybil_canister_address {
            self.sybil_canister_address = sybil_canister_address;
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(skip, default = "init_metadata")]
    pub metadata: StableCell<Cbor<Metadata>, VMemory>,

    #[serde(skip, default = "init_chains")]
    // TODO: change to u64
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
