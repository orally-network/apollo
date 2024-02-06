use std::collections::HashMap;

use crate::types::apollo_instance::ApolloInstance;
use apollo_utils::errors::ApolloError;
use apollo_utils::memory::Cbor;
use candid::Nat;

use types::{Metadata, STATE};
use utils::set_custom_panic_hook;

mod memory;
mod methods;
mod migrations;
mod types;
mod utils;

#[ic_cdk::init]
fn init(sybil_canister_address: String, key_name: String) {
    set_custom_panic_hook();

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state
            .metadata
            .set(Cbor(Metadata {
                key_name,
                sybil_canister_address,
            }))
            .unwrap();
    });
}

// For candid file auto-generation
pub type Result<T> = std::result::Result<T, ApolloError>;
use apollo_utils::apollo_instance::Metadata as ApolloInstanceMetadata;
use apollo_utils::apollo_instance::UpdateMetadata;
use types::candid_types::*;
// Candid file auto-generation
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
            .join("apollo");
        println!("{}", dir.to_str().unwrap());
        write(dir.join("apollo.did"), export_candid()).expect("Write failed.");
    }
}
