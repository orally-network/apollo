use std::collections::HashMap;

use candid::{candid_method, Nat, encode_one, encode_args};
use ic_cdk::{query, api::management_canister::main::{install_code, InstallCodeArgument, CanisterInstallMode, CreateCanisterArgument, create_canister}, update};
use ic_stable_structures::StableBTreeMap;
use memory::{Cbor, VMemory};
use types::{STATE, Metadata, ApolloIntanceMetadata};
use utils::set_custom_panic_hook;
use anyhow::Result;

use crate::{types::{errors::{ApolloError, ApolloInstanceError}, apollo_instance::ApolloInstance}, utils::nat::ToNativeTypes};

mod memory;
mod migrations;
mod utils;
mod types;

const INIT_CYCLES_BALANCE: u128 = 3_000_000_000_000; 


#[candid_method]
#[query]
fn get_metadata() -> Metadata {
    log!("Metadata");
    STATE.with(|s| {
        s.borrow().metadata.get().0.clone()
    })
}


#[candid_method]
#[query]
fn get_chain_metadata(chain_id: Nat) -> ApolloIntanceMetadata {
    log!("Chain Metadata");
    Default::default()
}

#[candid_method]
#[query]
fn get_chains() -> HashMap<u32, ApolloInstance> {
    log!("Chains");
    STATE.with(|s| {
        log!("ABOBA");
        s.borrow().chains.iter().map(|(k, v)| (k.clone(), v.0.clone())).collect()
    })
} 


#[candid_method]
#[update]
async fn add_chain(chain_id: Nat) -> Result<(), ApolloError> {
    log!("Adding Chain: {}", chain_id);
    if STATE.with(|state| state.borrow().chains.contains_key(&chain_id.to_u32())) {
        return Err(ApolloInstanceError::ChainAlreadyExists(chain_id).into());
    }


    let canister_id = match create_canister(CreateCanisterArgument { settings: None }, INIT_CYCLES_BALANCE.into()).await {
        Ok((canister_id,)) => canister_id.canister_id,
        Err((_, error)) => {
            return Err(ApolloInstanceError::FailedToCreate(error.to_string()).into());
        }
    };

    let wasm = include_bytes!("../../../apollo_instance.wasm").to_vec();

    let payload = (get_metadata!(tx_fee), get_metadata!(key_name), chain_id.clone());

    match install_code(InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: canister_id.clone(),
        wasm_module: wasm.to_vec(),
        arg: encode_args(payload).unwrap(),
    }).await {
        Ok(()) => {
            log!("Code installed in apollo instance with chain_id: {}", chain_id);
            STATE.with(|s| {
                let mut state = s.borrow_mut();
                let apollo_instance = ApolloInstance {
                    canister_id: canister_id.clone(),
                    is_active: true,
                };

                state.chains.insert(chain_id.to_u32(), Cbor(apollo_instance));
            });
        },
        Err((_, error)) => {
            return Err(ApolloInstanceError::FailedToInstallCode(error.to_string()).into());
        }
    }


    log!("Chain Added: {}", chain_id);
    Ok(())
}


#[ic_cdk::init]
fn init(tx_fee: Nat, key_name: String, timer_frequency: Nat) {
    set_custom_panic_hook();

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.metadata.set(Cbor(Metadata {
            tx_fee,
            key_name,
            timer_frequency,
            ..Default::default()
        })).unwrap();
    });
}


// For candid file auto-generation
candid::export_service!();
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
        let dir = dir.parent().unwrap().parent().unwrap().join("src").join("apollo");
        println!("{}", dir.to_str().unwrap());
        write(dir.join("apollo.did"), export_candid()).expect("Write failed.");
    }
}
