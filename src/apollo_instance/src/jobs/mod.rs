use apollo_utils::{
    address,
    errors::MulticallError,
    get_metadata, log,
    multicall::{self, Call},
    nat::{ToNatType, ToNativeTypes},
    sybil::{get_sybil_feed, AssetData},
    web3::Web3Instance,
};
use ic_web3_rs::{ethabi::Contract, types::U256, Transport};

use crate::{
    types::{allowances::Allowances, balances::Balances, timer::Timer, ApolloCoordinatorRequest},
    utils::apollo_evm_address,
};

use anyhow::Result;

mod logs_polling;
pub mod withdraw;

const TARGET_FUNCTIONS_ABI: &str = include_str!("../../../../assets/TargetFunctionsABI.json");

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

        withdraw::execute();
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
        let balance = Balances::get(&Allowances::get_allowed_user(address::from_h160(
            &apollo_coordinator_request.requester,
        ))?)?
        .amount;

        if balance
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
                balance
            );

            continue;
        }

        let sybil_feed = get_sybil_feed(
            get_metadata!(sybil_canister_address),
            apollo_coordinator_request.feed_id.clone(),
        )
        .await?;

        let contract_with_functions = Contract::load(TARGET_FUNCTIONS_ABI.as_bytes())?;

        let call_data = match sybil_feed {
            AssetData::DefaultPriceFeed { .. } | AssetData::CustomPriceFeed { .. } => {
                contract_with_functions
                    .function("update_feed")?
                    .encode_input(&sybil_feed.encode())
                    .map_err(|err| MulticallError::UnableToEncodeCallData(err.to_string()))?
            }
            AssetData::CustomNumber { .. } => contract_with_functions
                .function("update_custom_number")?
                .encode_input(&sybil_feed.encode())
                .map_err(|err| MulticallError::UnableToEncodeCallData(err.to_string()))?,
            AssetData::CustomString { .. } => contract_with_functions
                .function("update_custom_string")?
                .encode_input(&sybil_feed.encode())
                .map_err(|err| MulticallError::UnableToEncodeCallData(err.to_string()))?,
        };

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

        Balances::reduce_amount(
            &Allowances::get_allowed_user(format!("{:?}", call.target))?,
            &amount,
        )
        .expect("should reduce balance");
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_target_function_abi() -> Result<()> {
        let contract_with_functions = Contract::load(TARGET_FUNCTIONS_ABI.as_bytes())?;
        let _ = contract_with_functions.function("update_feed")?;
        let _ = contract_with_functions.function("update_custom_number")?;
        let _ = contract_with_functions.function("update_custom_string")?;

        Ok(())
    }
}
