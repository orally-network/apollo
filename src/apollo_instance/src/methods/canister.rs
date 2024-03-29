use crate::{types::STATE, utils::apollo_evm_address, Result};
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

    log!(
        "Updating timer frequency from {} to {} seconds",
        get_state!(timer_frequency_sec),
        timer_frequency_sec
    );

    update_state!(timer_frequency_sec, timer_frequency_sec);
    Ok(())
}

#[candid_method]
#[update]
async fn update_last_parsed_logs_from_block(block_number: Option<u64>) -> Result<()> {
    validate_caller()?;

    update_state!(last_parsed_logs_from_block, block_number);
    Ok(())
}

#[candid_method]
#[update]
async fn get_apollo_address() -> Result<String> {
    Ok(apollo_evm_address().await?)
}
