use apollo_utils::{
    apollo_instance::{Metadata as ApolloInstanceMetadata, UpdateMetadata},
    errors::{ApolloError, ApolloInstanceError},
    retry_until_success,
};
use candid::{candid_method, Nat};
use ic_cdk::update;

use crate::Result;

#[candid_method]
#[update]
async fn get_apollo_instance_metadata(chain_id: Nat) -> Result<ApolloInstanceMetadata> {
    let apollo_instance = crate::get_apollo_instance!(chain_id);

    let (result,): (ApolloInstanceMetadata,) = retry_until_success!(ic_cdk::call(
        apollo_instance.canister_id,
        "get_metadata",
        ()
    ))
    .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

    Ok(result)
}

#[candid_method]
#[update]
async fn update_apollo_instance_metadata(
    chain_id: Nat,
    update_metadata_args: UpdateMetadata,
) -> Result<()> {
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
async fn get_ama(chain_id: Nat) -> Result<String> {
    let apollo_instance = crate::get_apollo_instance!(chain_id);

    let (result,): (std::result::Result<String, ApolloInstanceError>,) = retry_until_success!(
        ic_cdk::call(apollo_instance.canister_id, "get_apollo_address", ())
    )
    .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

    Ok(result?)
}
