use crate::{jobs, types::STATE, utils::apollo_evm_address, Result};
use apollo_utils::{
    apollo_instance::{ApolloInstanceMetadata, UpdateMetadata},
    canister::validate_caller,
    get_state, log,
    memory::Cbor,
    update_state,
};
use candid::candid_method;
use ic_cdk::{query, update};

#[candid_method]
#[query]
fn get_metadata() -> ApolloInstanceMetadata {
    STATE.with(|s| s.borrow().metadata.get().0.clone())
}

#[candid_method]
#[update]
fn update_metadata(update_metadata_args: UpdateMetadata) -> Result<()> {
    validate_caller()?;

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let mut metadata = state.metadata.get().0.clone();
        metadata.update(update_metadata_args);
        state.metadata.set(Cbor(metadata)).unwrap();
    });

    Ok(())
}

#[candid_method]
#[update]
fn update_timer_frequency_sec(timer_frequency_sec: u64) -> Result<()> {
    validate_caller()?;

    update_state!(timer_frequency_sec, timer_frequency_sec);
    Ok(())
}

/// Set's last request id from apollo coordinator.
/// Automatically sets to max request_id from apollo coordinator if not provided.
#[candid_method]
#[update]
async fn update_last_request_id(request_id: Option<u64>) -> Result<()> {
    validate_caller()?;

    if let Some(request_id) = request_id {
        update_state!(last_request_id, Some(request_id));
    } else {
        // This func is automatically aborted by icp because of wait_success_confirmation func.
        // So, we just spawn a new async task to update last_request_id in the background.
        ic_cdk::spawn(async {
            log!("Current request id: {:?}", get_state!(last_request_id));
            if let Err(e) = jobs::apollo_coordinator_polling::update_last_request_id().await {
                log!("Error while executing publisher job: {e:?}");
            }
            log!("Updated request id: {:?}", get_state!(last_request_id));
        });
    }

    Ok(())
}

#[candid_method]
#[update]
async fn get_apollo_address() -> Result<String> {
    Ok(apollo_evm_address().await?)
}
