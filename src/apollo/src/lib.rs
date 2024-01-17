use std::collections::HashMap;

use candid::Nat;
use ic_cdk::api::management_canister::http_request::{TransformArgs, HttpResponse};
use types::{STATE, Metadata, candid_types::AddChainRequest};
use anyhow::Result;
use crate::types::apollo_instance::ApolloInstance;
use apollo_utils::errors::ApolloError;

mod memory;
mod migrations;
mod utils;
mod types;
mod methods;

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
