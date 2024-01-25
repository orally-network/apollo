use apollo_utils::log;
use candid::{candid_method, CandidType, Nat};
use ic_cdk::{query, update};
use jobs::apollo_coordinator_polling::_execute;
use memory::Cbor;
use serde::{Deserialize, Serialize};
use types::{Metadata, STATE};
use utils::{apollo_evm_address, set_custom_panic_hook};

use crate::types::timer::Timer;

mod jobs;
mod memory;
mod migrations;
mod types;
mod utils;

#[cfg(feature = "build_canister")]
#[candid_method]
#[query]
fn get_metadata() -> Metadata {
    STATE.with(|s| s.borrow().metadata.get().0.clone())
}

#[candid_method]
#[update]
async fn start() {
    // Timer::activate();
    // execute();
    _execute().await.unwrap();
}

#[candid_method]
#[update]
async fn get_apollo_address() -> String {
    apollo_evm_address().await.unwrap()
}

#[candid_method]
#[update]
fn stop() {
    Timer::deactivate().unwrap();
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct ApolloInstanceInit {
    apollos_fee: Nat,
    key_name: String,
    chain_id: Nat,
    chain_rpc: String,
    apollo_coordinator: String,
    timer_frequency: u64,
}

// Used to generate metadata from ApolloInstanceInit
impl From<ApolloInstanceInit> for Metadata {
    fn from(init: ApolloInstanceInit) -> Self {
        Self {
            apollos_fee: init.apollos_fee,
            key_name: init.key_name,
            chain_id: init.chain_id,
            chain_rpc: init.chain_rpc,
            apollo_coordinator: init.apollo_coordinator,
            apollo_evm_address: None,
        }
    }
}

#[ic_cdk::init]
fn init(args: ApolloInstanceInit) {
    set_custom_panic_hook();

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.timer_frequency = args.timer_frequency;
        state.metadata.set(Cbor(args.into())).unwrap();
    });
}

// For candid file auto-generation
#[cfg(feature = "build_canister")]
candid::export_service!();

/// Not a test, but a helper function to save the candid file
#[cfg(test)]
#[cfg(feature = "build_canister")]
mod save_candid {

    use super::*;

    fn export_candid() -> String {
        __export_service()
    }

    #[test]
    fn update_candid() {
        use std::env;
        use std::fs::write;
        use std::path::PathBuf;

        let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let dir = dir
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("src")
            .join("apollo_instance");
        println!("{}", dir.to_str().unwrap());
        write(dir.join("apollo_instance.did"), export_candid()).expect("Write failed.");
    }
}
