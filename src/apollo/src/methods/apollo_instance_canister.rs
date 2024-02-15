use apollo_utils::{
    apollo_instance::{ApolloInstanceMetadata, UpdateMetadata},
    canister::validate_caller,
    errors::{ApolloError, ApolloInstanceError},
    retry_until_success,
};
use candid::{candid_method, Nat};
use ic_cdk::update;

use crate::{types::custom_return_types::StringResult, ApolloInstanceMetadataResult, Result};

#[candid_method]
#[update]
async fn get_apollo_instance_metadata(chain_id: Nat) -> ApolloInstanceMetadataResult {
    let result = async move {
        let apollo_instance = crate::get_apollo_instance!(chain_id);

        let (result,): (ApolloInstanceMetadata,) = retry_until_success!(ic_cdk::call(
            apollo_instance.canister_id,
            "get_metadata",
            ()
        ))
        .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

        Ok(result)
    }
    .await;

    match result {
        Ok(metadata) => ApolloInstanceMetadataResult::Ok(metadata),
        Err(err) => ApolloInstanceMetadataResult::Err(err),
    }
}

#[candid_method]
#[update]
async fn update_apollo_instance_metadata(
    chain_id: Nat,
    update_metadata_args: UpdateMetadata,
) -> Result<()> {
    validate_caller()?;
    let apollo_instance = crate::get_apollo_instance!(chain_id);

    let (result,): (std::result::Result<(), ApolloInstanceError>,) =
        retry_until_success!(ic_cdk::call(
            apollo_instance.canister_id,
            "update_metadata",
            (update_metadata_args.clone(),)
        ))
        .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

    Ok(result?)
}

#[candid_method]
#[update]
async fn update_timer_frequency_sec(chain_id: Nat, timer_frequency_sec: u64) -> Result<()> {
    validate_caller()?;
    let apollo_instance = crate::get_apollo_instance!(chain_id);

    let (result,): (std::result::Result<(), ApolloInstanceError>,) =
        retry_until_success!(ic_cdk::call(
            apollo_instance.canister_id,
            "update_timer_frequency_sec",
            (timer_frequency_sec.clone(),)
        ))
        .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

    Ok(result?)
}

#[candid_method]
#[update]
async fn get_ama(chain_id: Nat) -> StringResult {
    let result: Result<String> = async move {
        let apollo_instance = crate::get_apollo_instance!(chain_id);

        let (result,): (std::result::Result<String, ApolloInstanceError>,) = retry_until_success!(
            ic_cdk::call(apollo_instance.canister_id, "get_apollo_address", ())
        )
        .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

        Ok(result?)
    }
    .await;

    match result {
        Ok(address) => StringResult::Ok(address),
        Err(err) => StringResult::Err(err),
    }
}
