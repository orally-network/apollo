use apollo_utils::log;

use crate::{jobs::apollo_coordinator_polling::_execute, types::timer::Timer};

pub mod apollo_coordinator_polling;

pub fn execute() {
    log!("---Execution started---");

    ic_cdk::spawn(async {
        if let Err(e) = _execute().await {
            log!("Error while executing publisher job: {e:?}");
        } else {
            log!("Publisher job executed successfully");
        }

        // if Timer::is_active() {
        //     Timer::set_timer(execute);
        // }
    });
}
