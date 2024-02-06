use apollo_utils::{errors::ApolloError, memory::Cbor, retry_until_success};
use candid::{candid_method, Nat};
use ic_cdk::update;

use crate::{update_apollo_instance, Result};

#[candid_method]
#[update]
async fn start(chain_id: Nat) -> Result<()> {
    let mut apollo_instance = crate::get_apollo_instance!(chain_id.clone());
    apollo_instance.is_active = true;

    update_apollo_instance!(chain_id.clone(), apollo_instance.clone());

    retry_until_success!(ic_cdk::call(apollo_instance.canister_id, "start", ()))
        .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

    Ok(())
}

#[candid_method]
#[update]
async fn stop(chain_id: Nat) -> Result<()> {
    let mut apollo_instance = crate::get_apollo_instance!(chain_id.clone());
    apollo_instance.is_active = false;

    update_apollo_instance!(chain_id.clone(), apollo_instance.clone());

    retry_until_success!(ic_cdk::call(apollo_instance.canister_id, "stop", ()))
        .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

    Ok(())
}
