use apollo_utils::canister::validate_caller;
use apollo_utils::errors::ApolloInstanceError;
use candid::candid_method;
use ic_cdk::update;

use crate::Result;
use crate::{jobs::execute, types::timer::Timer};

#[candid_method]
#[update]
pub fn start() -> Result<()> {
    validate_caller()?;

    if Timer::is_active() {
        Timer::deactivate()
            .map_err(|err| ApolloInstanceError::FailedToRestartTimer(err.to_string()))?;
    }
    Timer::activate();
    execute();

    Ok(())
}

#[candid_method]
#[update]
pub fn stop() -> Result<()> {
    validate_caller()?;

    Timer::deactivate().unwrap();

    Ok(())
}
