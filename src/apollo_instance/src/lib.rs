use apollo_utils::{
    address,
    errors::{ApolloInstanceError, BalancesError, Web3Error},
    get_metadata, log, macros,
    nat::ToNatType,
    web3,
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

/// Deposit amount to the AMA
///
/// # Arguments
///
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `msg` - SIWE message, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
/// * `sig` - SIWE signature, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
///
/// # Returns
///
/// Returns a result that can contain an error message
#[candid_method]
#[update]
pub async fn deposit(tx_hash: String, msg: String, sig: String) -> Result<()> {
    let address = apollo_utils::siwe::recover(msg, sig).await;

    let w3 = web3::instance(&get_metadata!(chain_rpc))?;

    let tx = w3.get_tx(&tx_hash).await?;

    let receiver = tx.to.ok_or(Web3Error::TxWithoutReceiver)?;

    let ama = address::to_h160(&apollo_evm_address().await?)?;

    if receiver != ama {
        return Err(ApolloInstanceError::TxWasNotSentToAMA);
    }

    Balances::save_nonce(&address, &tx.nonce.to_nat())?;

    let amount = tx.value.to_nat();

    Balances::add_amount(&address, &amount)?;

    log!("[BALANCES] {address} deposited amount {amount}");
    Ok(())
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
    sybil_canister_address: String,
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
