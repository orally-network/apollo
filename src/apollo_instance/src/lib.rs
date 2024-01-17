use std::{borrow::BorrowMut, time::Duration};
use anyhow::Result;
use apollo_utils::{log, get_state, update_state, errors::ApolloInstanceError};
use candid::{candid_method, Nat};
use ic_cdk::{query, update, api::management_canister::http_request::{HttpResponse, TransformArgs}};
use ic_cdk_timers::set_timer;
use ic_web3_rs::{Web3, transports::{ICHttp, ic_http_client::CallOptions}, types::{BlockId, BlockNumber, Call}};
use memory::Cbor;
use types::{Metadata, STATE};
use utils::set_custom_panic_hook;

use crate::types::timer::Timer;

mod memory;
mod migrations;
mod utils;
mod types;


const BLOCK_NUMBER_THRESHOLD: u64 = 100; // Block with height difference more than this will be considered as old and won't be processed



#[cfg(feature = "build_canister")]
#[candid_method]
#[query]
fn get_metadata() -> Metadata {
    STATE.with(|s| {
        s.borrow().metadata.get().0.clone()
    })
}

#[query]
fn transform(response: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: response.response.status,
        body: response.response.body,
        headers: Vec::new(),
    }
}

#[candid_method]
#[update]
fn start() {
    Timer::activate();
    execute();
}

#[candid_method]
#[update]
fn stop() {
    Timer::deactivate().unwrap();
}



fn execute() {
    ic_cdk::spawn(async {
        if let Err(e) = _execute().await {
            log!("Error while executing publisher job: {e:?}");
        }
    });
}



async fn _execute() -> Result<()> {
    if Timer::is_active() {
        return Ok(());
    }

    log!("Watch blocks started. {:#?}", get_state!(last_checked_block_height));

    let w3 = Web3::new(ICHttp::new("https://goerli.infura.io/v3/8e4147cd4995430182a09781136f8745", None).unwrap());

    let val = w3.eth().block(BlockId::Number(BlockNumber::Latest), CallOptions::default()).await;

    let blocks = vec![val.unwrap().unwrap()]; // TODO: Remove unwrap
    let last_block_height = blocks[0].number.unwrap().0[0];

    if let Some(last_checked_block_height) = get_state!(last_checked_block_height) {
        if last_block_height - last_checked_block_height > BLOCK_NUMBER_THRESHOLD {
            update_state!(last_checked_block_height, Some(last_block_height));
        }

        for i in (last_checked_block_height+1..last_block_height+1).rev() {
            let block = w3.eth().block(BlockId::Number(i.into()), CallOptions::default()).await.unwrap().unwrap(); // TODO: Remove unwrap
            let block_height = block.number.unwrap().0[0];

            update_state!(last_checked_block_height, Some(block_height));

            log!("Block: {:#?}", block.number);
        }
    }  else {
        update_state!(last_checked_block_height, Some(last_block_height));
    }


    let timer_id = set_timer(Duration::from_secs(get_state!(timer_frequency)), execute);
    Timer::update(timer_id)?;


    log!("Watch blocks finished");
    Ok(())
}


#[cfg(feature = "build_canister")]
#[ic_cdk::init]
fn init(tx_fee: Nat, key_name: String, chain_id: Nat, chain_rpc: String, timer_frequency: u64) {
    set_custom_panic_hook();

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.metadata.set(Cbor(Metadata {
            tx_fee,
            key_name,
            chain_id,
            chain_rpc,
        })).unwrap();

        state.timer_frequency = timer_frequency;
    });
}




// For candid file auto-generation
#[cfg(feature = "build_canister")]
candid::export_service!();

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
        let dir = dir.parent().unwrap().parent().unwrap().join("src").join("apollo_instance");
        println!("{}", dir.to_str().unwrap());
        write(dir.join("apollo_instance.did"), export_candid()).expect("Write failed.");
    }
}
