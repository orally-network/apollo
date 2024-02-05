use std::cell::RefCell;

use apollo_utils::log;
use candid::{CandidType, Nat};
use ic_stable_structures::StableCell;
use ic_web3_rs::{
    contract::{tokens::Tokenizable, Error},
    ethabi::Token,
    types::{H160, U256},
};
use serde::{Deserialize, Serialize};

use crate::memory::{Cbor, VMemory};

use self::{balances::Balances, timer::Timer};

pub mod balances;
pub mod timer;

#[derive(Serialize, Debug, Deserialize, CandidType, Clone)]
pub struct Metadata {
    pub apollos_fee: Nat,
    pub key_name: String,
    pub chain_id: Nat,
    pub chain_rpc: String,
    pub apollo_coordinator: String,
    pub apollo_evm_address: Option<String>,
    pub multicall_address: String,
    pub sybil_canister_address: String, // Principal is not supported by ciborium
    pub block_gas_limit: Nat,
    pub min_balance: Nat,
}

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct UpdateMetadata {
    pub apollos_fee: Option<Nat>,
    pub chain_id: Option<Nat>,
    pub chain_rpc: Option<String>,
    pub apollo_coordinator: Option<String>,
    pub multicall_address: Option<String>,
    pub sybil_canister_address: Option<String>, // Principal is not supported by ciborium
    pub block_gas_limit: Option<Nat>,
    pub min_balance: Option<Nat>,
}

impl Metadata {
    pub fn update(&mut self, update: UpdateMetadata) {
        if let Some(apollos_fee) = update.apollos_fee {
            self.apollos_fee = apollos_fee;
        }
        if let Some(chain_id) = update.chain_id {
            self.chain_id = chain_id;
        }
        if let Some(chain_rpc) = update.chain_rpc {
            self.chain_rpc = chain_rpc;
        }
        if let Some(apollo_coordinator) = update.apollo_coordinator {
            self.apollo_coordinator = apollo_coordinator;
        }
        if let Some(multicall_address) = update.multicall_address {
            self.multicall_address = multicall_address;
        }
        if let Some(sybil_canister_address) = update.sybil_canister_address {
            self.sybil_canister_address = sybil_canister_address;
        }
        if let Some(block_gas_limit) = update.block_gas_limit {
            self.block_gas_limit = block_gas_limit;
        }
        if let Some(min_balance) = update.min_balance {
            self.min_balance = min_balance;
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            apollos_fee: Nat::from(0),
            key_name: "".to_string(),
            chain_id: Nat::from(0),
            chain_rpc: "".to_string(),
            apollo_coordinator: "".to_string(),
            apollo_evm_address: None,
            multicall_address: "".to_string(),
            sybil_canister_address: "".to_string(),
            block_gas_limit: Nat::from(0),
            min_balance: Nat::from(0),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(skip, default = "init_metadata")]
    pub metadata: StableCell<Cbor<Metadata>, VMemory>, // TODO: move out to a separate Metadata struct

    #[serde(skip)]
    pub balances: Balances,

    // Frequency in seconds to check apollo coordinator for new requests
    pub timer_frequency: u64,
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
            timer_frequency: 0,
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
