use candid::{candid_method, Nat};
use ic_cdk::query;
use memory::Cbor;
use types::{Metadata, STATE};
use utils::set_custom_panic_hook;



mod memory;
mod migrations;
mod utils;
pub mod types;



#[cfg(feature = "build_canister")]
#[candid_method]
#[query]
fn get_metadata() -> Metadata {
    STATE.with(|s| {
        s.borrow().metadata.get().0.clone()
    })
}


#[cfg(feature = "build_canister")]
#[ic_cdk::init]
fn init(tx_fee: Nat, key_name: String, chain_id: Nat) {
    set_custom_panic_hook();

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.metadata.set(Cbor(Metadata {
            tx_fee,
            key_name,
            chain_id
        })).unwrap();
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
