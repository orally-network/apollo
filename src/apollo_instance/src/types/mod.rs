use std::cell::RefCell;

use candid::{CandidType, Nat};
use ic_stable_structures::StableCell;
use serde::{Serialize, Deserialize};

use crate::memory::{Cbor, VMemory};

use self::timer::Timer;

pub mod timer;

#[derive(Serialize, Deserialize, Default, CandidType, Clone)]
pub struct Metadata {
    pub tx_fee: Nat,
    pub key_name: String,
    pub chain_id: Nat,
    pub chain_rpc: String,
}

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(skip, default = "init_metadata")]
    pub metadata: StableCell<Cbor<Metadata>, VMemory>,

    pub last_checked_block_height: Option<u64>,
    // Frequency in seconds to check for new blocks
    pub timer_frequency: u64, 
    pub timer: Timer
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
            last_checked_block_height: None,
            timer_frequency: 0,
            timer: Timer::default(),
        }
    }
}