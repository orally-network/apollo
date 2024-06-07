use apollo_utils::{
    apollo_instance::{ApolloInstanceMetadata, UpdateMetadata},
    canister::validate_caller,
    errors::{ApolloError, ApolloInstanceError},
    nat::ToNativeTypes,
    retry_until_success,
};
use candid::{candid_method, Nat, Principal};
use ic_cdk::update;

use crate::{
    types::{custom_return_types::StringResult, STATE},
    ApolloInstanceMetadataResult, Result,
};

#[candid_method]
#[update]
pub async fn send_cycles(chain_id: Nat, destination: Principal, amount: Nat) -> Result<()> {
    validate_caller()?;
    let apollo_instance = crate::get_apollo_instance!(chain_id);

    let (result,): (std::result::Result<(), ApolloInstanceError>,) =
        retry_until_success!(ic_cdk::call(
            apollo_instance.canister_id,
            "send_cycles",
            (destination, amount.clone())
        ))
        .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

    Ok(result?)
}

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
    let apollo_instance = crate::get_apollo_instance!(chain_id.clone());

    let (result,): (std::result::Result<(), ApolloInstanceError>,) =
        retry_until_success!(ic_cdk::call(
            apollo_instance.canister_id,
            "update_metadata",
            (update_metadata_args.clone(),)
        ))
        .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

    // TODO: DELETE
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let mut chain = state.chains.get(&chain_id.to_u32()).unwrap();

        if let Some(apollo_coordinator) = update_metadata_args.apollo_coordinator {
            chain.0.apollo_coordinator = apollo_coordinator;
        }

        state.chains.insert(chain_id.to_u32(), chain);
    });

    Ok(result?)
}

#[candid_method]
#[update]
async fn update_last_parsed_logs_from_block(
    chain_id: Nat,
    block_number: Option<u64>,
) -> Result<()> {
    validate_caller()?;
    let apollo_instance = crate::get_apollo_instance!(chain_id);

    let (result,): (std::result::Result<(), ApolloInstanceError>,) =
        retry_until_success!(ic_cdk::call(
            apollo_instance.canister_id,
            "update_last_parsed_logs_from_block",
            (block_number.clone(),)
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
