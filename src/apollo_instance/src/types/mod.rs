use std::cell::RefCell;

use apollo_utils::{apollo_instance::Metadata, memory::Cbor};
use ic_stable_structures::StableCell;
use ic_web3_rs::{
    contract::{tokens::Tokenizable, Error},
    ethabi::Token,
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
    pub metadata: StableCell<Cbor<Metadata>, VMemory>, // TODO: move out to a separate Metadata struct

    #[serde(skip)]
    pub balances: Balances,

    // Frequency in seconds to check apollo coordinator for new requests
    pub timer_frequency_sec: u64,
    pub timer: Timer,
    pub last_request_id: u64,
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
            balances: Balances::default(),
            timer_frequency_sec: 0,
            timer: Timer::default(),
            last_request_id: 0,
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
