use apollo_utils::{
    address,
    errors::MulticallError,
    get_metadata, log,
    multicall::{self, Call},
    nat::{ToNatType, ToNativeTypes},
    sybil::get_sybil_input,
    web3::Web3Instance,
};
use ic_web3_rs::{ethabi::Function, types::U256, Transport};

use crate::{
    types::{balances::Balances, timer::Timer, ApolloCoordinatorRequest},
    utils::apollo_evm_address,
};

use anyhow::Result;

pub mod apollo_coordinator_polling;
mod logs_polling;

const TARGET_FUNCTION_ABI: &str = include_str!("../../../../assets/TargetFunctionABI.json");

pub fn execute() {
    if !Timer::is_active() {
        return;
    }

    log!("---Execution started---");

    ic_cdk::spawn(async {
        if let Err(e) = logs_polling::_execute().await {
            log!("Error while executing publisher job: {e:?}");
        } else {
            log!("Publisher job executed successfully");
        }

        Timer::set_timer(execute);
    });
}

async fn process_requests<T: Transport>(
    w3: &Web3Instance<T>,
    requests: Vec<ApolloCoordinatorRequest>,
    gas_price: U256,
) -> Result<()> {
    if requests.is_empty() {
        log!("[EXECUTION] No requests found");
        return Ok(());
    }

    log!("[EXECUTION] Processing {} requests", requests.len());

    let mut calls = Vec::with_capacity(requests.len());

    for apollo_coordinator_request in requests {
        // TODO: implement balance check
        if Balances::get(&address::from_h160(&apollo_coordinator_request.requester))?.amount
            < get_metadata!(min_balance)
                + apollo_coordinator_request.callback_gas_limit.to_nat()
                + get_metadata!(apollos_fee)
        {
            log!(
                "[EXECUTION] chain: {}, not enough balance for requester {}. Needed (min_balance + callback_gas_limit + apollos_fee): {} + {} + {} = {}, current: {}",
                get_metadata!(chain_id),
                apollo_coordinator_request.requester,
                get_metadata!(min_balance),
                apollo_coordinator_request.callback_gas_limit,
                get_metadata!(apollos_fee),
                get_metadata!(min_balance) + apollo_coordinator_request.callback_gas_limit.to_nat() + get_metadata!(apollos_fee),
                Balances::get(&address::from_h160(&apollo_coordinator_request.requester))?.amount
            );

            continue;
        }

        let target_func = serde_json::from_str::<Function>(TARGET_FUNCTION_ABI)?;
        let call_data = target_func
            .encode_input(
                &get_sybil_input(
                    get_metadata!(sybil_canister_address),
                    apollo_coordinator_request.feed_id,
                )
                .await?,
            )
            .map_err(|err| MulticallError::UnableToEncodeCallData(err.to_string()))?;

        calls.push(Call {
            target: apollo_coordinator_request.requester,
            call_data,
            gas_limit: apollo_coordinator_request.callback_gas_limit,
        });
    }

    let results = multicall::multicall(
        w3,
        &get_metadata!(multicall_address),
        apollo_evm_address().await?,
        calls.clone(),
        get_metadata!(key_name),
        get_metadata!(chain_id).to_u64(),
        get_metadata!(block_gas_limit).to_u256(),
        &gas_price,
    )
    .await?;

    // TODO: reimplement
    for (result, call) in results.iter().zip(calls) {
        log!(
            "[EXECUTION] chain: {}, requester: {}, used gas: {}, gas limit: {}",
            get_metadata!(chain_id),
            call.target,
            result.used_gas,
            call.gas_limit
        );

        #[allow(clippy::cmp_owned)]
        if result.used_gas == 0.into() {
            panic!("used_gas is 0"); //TODO: check and remove
        }

        if result.used_gas > call.gas_limit {
            panic!("used_gas is greater than gas_limit"); //TODO: check and remove
        }

        let amount = gas_price.to_nat() * result.used_gas.to_nat() + get_metadata!(apollos_fee);

        Balances::reduce_amount(&format!("{:?}", call.target), &amount)
            .expect("should reduce balance");
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_target_function_abi() -> Result<()> {
        let _ = serde_json::from_str::<Function>(TARGET_FUNCTION_ABI)?;

        Ok(())
    }
}
