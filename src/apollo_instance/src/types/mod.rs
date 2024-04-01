use std::cell::RefCell;

use crate::memory::VMemory;
use apollo_utils::{apollo_instance::ApolloInstanceMetadata, memory::Cbor};
use ic_stable_structures::StableCell;
use ic_web3_rs::{
    ethabi::Log,
    types::{H160, U256},
};
use serde::{Deserialize, Serialize};

use self::{allowances::Allowances, balances::Balances, timer::Timer, withdraw::WithdrawRequests};

pub mod allowances;
pub mod asset_data;
pub mod balances;
pub mod timer;
pub mod withdraw;

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(skip, default = "init_metadata")]
    pub metadata: StableCell<Cbor<ApolloInstanceMetadata>, VMemory>, // TODO: move out to a separate Metadata struct

    #[serde(skip)]
    pub balances: Balances,

    #[serde(skip)]
    pub withdraw_requests: WithdrawRequests,

    #[serde(skip)]
    pub allowances: Allowances,

    // Frequency in seconds to check apollo coordinator for new requests
    pub timer_frequency_sec: u64,
    pub timer: Timer,
    // last parsed block for logs_polling
    pub last_parsed_logs_from_block: Option<u64>,
}

thread_local! {
    pub static STATE: RefCell<State> = RefCell::new(State::default());
}

fn init_metadata() -> StableCell<Cbor<ApolloInstanceMetadata>, VMemory> {
    let metadata = Cbor(ApolloInstanceMetadata::default());
    StableCell::init(crate::memory::get_metadata_memory(), metadata).unwrap()
}

impl Default for State {
    fn default() -> Self {
        Self {
            metadata: init_metadata(),
            balances: Balances::default(),
            withdraw_requests: WithdrawRequests::default(),
            allowances: Allowances::default(),
            timer_frequency_sec: 0,
            timer: Timer::default(),
            last_parsed_logs_from_block: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ApolloCoordinatorRequest {
    DataFeed {
        request_id: U256,
        feed_id: String,
        callback_gas_limit: U256,
        requester: H160,
    },
    RandomFeed {
        request_id: U256,
        callback_gas_limit: U256,
        num_words: U256,
        requester: H160,
    },
}

impl ApolloCoordinatorRequest {
    pub fn feed_id(&self) -> String {
        match self {
            Self::DataFeed { feed_id, .. } => feed_id.clone(),
            Self::RandomFeed { .. } => "random".to_string(),
        }
    }

    pub fn callback_gas_limit(&self) -> U256 {
        match self {
            Self::DataFeed {
                callback_gas_limit, ..
            } => callback_gas_limit.clone(),
            Self::RandomFeed {
                callback_gas_limit, ..
            } => callback_gas_limit.clone(),
        }
    }

    pub fn requester(&self) -> H160 {
        match self {
            Self::DataFeed { requester, .. } => requester.clone(),
            Self::RandomFeed { requester, .. } => requester.clone(),
        }
    }

    pub fn new_from_data_feed_log(log: Log) -> Self {
        let params = log.params;

        let request_id = params
            .get(0)
            .expect("should be able to get request_id from log")
            .value
            .clone()
            .into_uint()
            .expect("should be able to convert to uint");

        let feed_id = params
            .get(1)
            .expect("should be able to get feed_id from log")
            .value
            .clone()
            .into_string()
            .expect("should be able to convert to string");

        let callback_gas_limit = params
            .get(2)
            .expect("should be able to get callback_gas_limit from log")
            .value
            .clone()
            .into_uint()
            .expect("should be able to convert to uint");

        let requester = params
            .get(3)
            .expect("should be able to get requester from log")
            .value
            .clone()
            .into_address()
            .expect("should be able to convert to address");

        Self::DataFeed {
            request_id,
            feed_id,
            callback_gas_limit,
            requester,
        }
    }

    pub fn new_from_random_feed_log(log: Log) -> Self {
        let params = log.params;

        let request_id = params
            .get(0)
            .expect("should be able to get request_id from log")
            .value
            .clone()
            .into_uint()
            .expect("should be able to convert to uint");

        let callback_gas_limit = params
            .get(1)
            .expect("should be able to get callback_gas_limit from log")
            .value
            .clone()
            .into_uint()
            .expect("should be able to convert to uint");

        let num_words = params
            .get(2)
            .expect("should be able to get num_words from log")
            .value
            .clone()
            .into_uint()
            .expect("should be able to convert to uint");

        let requester = params
            .get(3)
            .expect("should be able to get requester from log")
            .value
            .clone()
            .into_address()
            .expect("should be able to convert to address");

        Self::RandomFeed {
            request_id,
            callback_gas_limit,
            num_words,
            requester,
        }
    }
}
