use candid::Principal;
use ic_cdk::{api::is_controller, query, update};
use ic_web3_rs::{
    ethabi::Address,
    ic::{get_public_key, pubkey_to_address},
};

use crate::errors::UtilsError;

/// get canister's eth address
/// TODO: delete ?
pub async fn get_eth_addr(
    canister_id: Option<Principal>,
    derivation_path: Option<Vec<Vec<u8>>>,
    name: String,
) -> Result<Address, String> {
    let path = if let Some(v) = derivation_path {
        v
    } else {
        vec![ic_cdk::id().as_slice().to_vec()]
    };

    match get_public_key(canister_id, path, name).await {
        Ok(pubkey) => pubkey_to_address(&pubkey),
        Err(e) => Err(e),
    }
}

pub fn validate_caller() -> Result<(), UtilsError> {
    if is_controller(&ic_cdk::caller()) {
        return Ok(());
    }

    Err(UtilsError::NotAController)
}

fn validate_canistergeek_caller() {
    match Principal::from_text("hozae-racaq-aaaaa-aaaaa-c") {
        Ok(caller) if caller == ic_cdk::caller() => (),
        _ => ic_cdk::trap("Invalid caller"),
    }
}

#[query(name = "getCanistergeekInformation")]
pub async fn get_canistergeek_information(
    request: ic_utils::api_type::GetInformationRequest,
) -> ic_utils::api_type::GetInformationResponse<'static> {
    ic_utils::get_information(request)
}

#[update(name = "updateCanistergeekInformation")]
pub async fn update_canistergeek_information(
    request: ic_utils::api_type::UpdateInformationRequest,
) -> () {
    validate_canistergeek_caller();
    ic_utils::update_information(request);
}
