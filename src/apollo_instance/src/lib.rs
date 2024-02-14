use apollo_utils::memory::Cbor;
use apollo_utils::{apollo_instance::ApolloInstanceInit, errors::ApolloInstanceError, log};
use candid::Nat;
use types::STATE;
use utils::set_custom_panic_hook;

mod jobs;
mod memory;
mod methods;
mod migrations;
mod types;
mod utils;

#[ic_cdk::init]
fn init(args: ApolloInstanceInit) {
    set_custom_panic_hook();

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.timer_frequency_sec = args.timer_frequency_sec;
        state.metadata.set(Cbor(args.into())).unwrap();
    });
}

// For candid file auto-generation
pub type Result<T> = std::result::Result<T, ApolloInstanceError>;
pub type NatResult = std::result::Result<Nat, ApolloInstanceError>;
pub type StringResult = std::result::Result<String, ApolloInstanceError>;

use apollo_utils::apollo_instance::ApolloInstanceMetadata;
use apollo_utils::apollo_instance::UpdateMetadata;

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
