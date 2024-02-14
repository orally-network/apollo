use std::cell::RefCell;

use apollo_utils::{apollo_instance::ApolloInstanceMetadata, memory::Cbor};
use ic_stable_structures::StableCell;
use ic_web3_rs::{
    contract::{tokens::Tokenizable, Error},
    ethabi::{Log, Token},
    types::{H160, U256},
};
use serde::{Deserialize, Serialize};

use crate::memory::VMemory;

use self::{balances::Balances, timer::Timer};

pub mod balances;
pub mod timer;

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(skip, default = "init_metadata")]
    pub metadata: StableCell<Cbor<ApolloInstanceMetadata>, VMemory>, // TODO: move out to a separate Metadata struct

    #[serde(skip)]
    pub balances: Balances,

    // Frequency in seconds to check apollo coordinator for new requests
    pub timer_frequency_sec: u64,
    pub timer: Timer,
    // last request id for apollo_coordinator_polling
    pub last_request_id: Option<u64>,
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
            timer_frequency_sec: 0,
            timer: Timer::default(),
            last_request_id: None,
            last_parsed_logs_from_block: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ApolloCoordinatorRequest {
    pub request_id: U256,
    pub feed_id: String,
    pub callback_gas_limit: U256,
    pub requester: H160,
}

impl From<Log> for ApolloCoordinatorRequest {
    fn from(log: Log) -> Self {
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

        Self {
            request_id,
            feed_id,
            callback_gas_limit,
            requester,
        }
    }
}

impl Tokenizable for ApolloCoordinatorRequest {
    fn from_token(token: Token) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Token::Tuple(tokens) = token {
            if tokens.len() != 4 {
                return Err(Error::InvalidOutputType("invalid tokens number".into()));
            }
            if let [Token::Uint(request_id), Token::String(feed_id), Token::Uint(callback_gas_limit), Token::Address(requester)] =
                tokens.as_slice()
            {
                return Ok(Self {
                    request_id: request_id.clone(),
                    feed_id: feed_id.clone(),
                    callback_gas_limit: callback_gas_limit.clone(),
                    requester: requester.clone(),
                });
            }
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            Token::Uint(self.request_id),
            Token::String(self.feed_id),
            Token::Uint(self.callback_gas_limit),
            Token::Address(self.requester),
        ])
    }
}
