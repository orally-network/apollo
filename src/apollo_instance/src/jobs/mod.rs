use apollo_utils::log;

use self::apollo_coordinator_polling::_execute;

pub mod apollo_coordinator_polling;

pub fn execute() {
    ic_cdk::spawn(async {
        if let Err(e) = _execute().await {
            log!("Error while executing publisher job: {e:?}");
        }
    });
}
