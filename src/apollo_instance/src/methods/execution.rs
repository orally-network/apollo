use candid::candid_method;
use ic_cdk::update;

use crate::{jobs::execute, types::timer::Timer};

#[candid_method]
#[update]
pub fn start() {
    Timer::activate();
    execute();
}

#[candid_method]
#[update]
pub fn stop() {
    Timer::deactivate().unwrap();
}
