use apollo_utils::canister::validate_caller;
use candid::candid_method;
use ic_cdk::update;

use crate::Result;
use crate::{jobs::execute, types::timer::Timer};

#[candid_method]
#[update]
pub fn start() -> Result<()> {
    validate_caller()?;

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
