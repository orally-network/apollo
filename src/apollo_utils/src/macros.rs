#![allow(clippy::crate_in_macro_def)]

#[macro_export]
macro_rules! get_metadata {
    ($field:ident) => {{
        crate::STATE.with(|state| state.borrow().metadata.get().0.$field.clone())
    }};
}

#[macro_export]
macro_rules! update_metadata {
    ($field:ident, $value:expr) => {{
        crate::STATE.with(|state| {
            let mut metadata = state.borrow_mut().metadata.get().clone();
            (*metadata).$field = $value;
        });
    }};
}

#[macro_export]
macro_rules! get_state {
    ($field:ident) => {{
        crate::STATE.with(|state| state.borrow().$field.clone())
    }};
}

#[macro_export]
macro_rules! update_state {
    ($field:ident, $value:expr) => {{
        crate::STATE.with(|state| {
            state.borrow_mut().$field = $value;
        })
    }};
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        // use crate::metrics;
        ic_cdk::println!($($arg)*);
        ic_utils::logger::log_message(format!($($arg)*));
        ic_utils::monitor::collect_metrics();

        // metrics!(set CYCLES, ic_cdk::api::canister_balance() as u128);
    }};
}

#[macro_export]
macro_rules! retry_until_success {
    ($func:expr) => {
        retry_until_success!($func, 0)
    };

    ($func:expr, $chain_id:expr) => {{
        const MAX_RETRIES: u32 = 5;
        const DURATION_BETWEEN_ATTEMPTS: std::time::Duration = std::time::Duration::from_millis(1000);

        let mut attempts = 0u32;
        let mut result = $func.await;

        if $chain_id  != 0 {
            ic_utils::logger::log_message(format!("result in chain {}, retry_until_success: {:?}", $chain_id, result));
        } else {
            ic_utils::logger::log_message(format!("result in retry_until_success: {:?}", result));
        }

        let (func_name, func_other) = stringify!($func).rsplit_once("(").unwrap();

        while result.is_err()
            && (format!("{:?}", result.as_ref().unwrap_err()).contains("Canister http responses were different across replicas")
            || format!("{:?}", result.as_ref().unwrap_err()).contains("Timeout expired")
            || format!("{:?}", result.as_ref().unwrap_err()).contains("SysTransient")
            || format!("{:?}", result.as_ref().unwrap_err()).contains("pending") // or Exchange rate canister error: pending
            || format!("{:?}", result.as_ref().unwrap_err()).contains("No response")
            || format!("{:?}", result.as_ref().unwrap_err()).contains("already known"))
            && attempts < MAX_RETRIES
        {
            $crate::time::sleep(DURATION_BETWEEN_ATTEMPTS).await;
            result = $func.await;
            ic_utils::logger::log_message(format!("[{func_name} : {func_other}] attempt: {attempts}"));
            attempts += 1;
        }



        result
    }};
}
