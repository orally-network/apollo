use apollo_utils::{
    errors::{ApolloInstanceError, BalancesError},
    get_metadata, log, macros, web3,
};
use candid::{candid_method, CandidType, Nat, Principal};
use ic_cdk::{query, update};
use jobs::execute;
use memory::Cbor;
use serde::{Deserialize, Serialize};
use types::{balances::Balances, Metadata, STATE};
use utils::{apollo_evm_address, set_custom_panic_hook};

use crate::types::timer::Timer;

mod jobs;
mod memory;
mod migrations;
mod types;
mod utils;

type Result<T> = std::result::Result<T, ApolloInstanceError>;

#[candid_method]
#[query]
fn get_metadata() -> Metadata {
    STATE.with(|s| s.borrow().metadata.get().0.clone())
}

#[candid_method]
#[update]
async fn start() {
    Timer::activate();
    execute();
    // _execute().await.unwrap();
    // apollo_coordinator_polling::test().await.unwrap();
}

#[candid_method]
#[update]
fn stop() {
    Timer::deactivate().unwrap();
}

#[candid_method]
#[update]
async fn test_balances() -> Result<String> {
    // Balances::create("0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd")?;
    log!(
        "is exists: {}",
        Balances::is_exists("0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd")?
    );
    // Balances::save_nonce("0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd", &Nat::from(1))?;

    Balances::add_amount(
        "0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd",
        &Nat::from(1234567890),
    )?;

    let user_balances = Balances::get("0x89A4e2Cf7F72b6e462bbA27FEa4d40c3da1d46cd")?;
    Ok(format!("{:?}", user_balances))

    // let w3 = web3::instance(&get_metadata!(chain_rpc))?;
    // let balance = w3
    //     .get_address_balance(&apollo_evm_address().await.unwrap())
    //     .await?;

    // Ok(format!("{}", balance))
}

#[candid_method]
#[update]
async fn get_apollo_address() -> String {
    apollo_evm_address().await.unwrap()
}

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
struct ApolloInstanceInit {
    apollos_fee: Nat,
    key_name: String,
    chain_id: Nat,
    chain_rpc: String,
    apollo_coordinator: String,
    multicall_address: String,
    timer_frequency: u64,
    block_gas_limit: Nat,
    sybil_canister_address: Principal,
    min_balance: Nat,
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
            multicall_address: init.multicall_address,
            block_gas_limit: init.block_gas_limit,
            sybil_canister_address: init.sybil_canister_address,
            min_balance: init.min_balance,
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
candid::export_service!();

/// Not a test, but a helper function to save the candid file
#[cfg(test)]
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
