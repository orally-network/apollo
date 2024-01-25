use std::collections::HashMap;

use crate::types::apollo_instance::ApolloInstance;
use anyhow::Result;
use apollo_utils::errors::ApolloError;
use candid::Nat;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use types::{candid_types::AddChainRequest, Metadata, STATE};

mod memory;
mod methods;
mod migrations;
mod types;
mod utils;

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
