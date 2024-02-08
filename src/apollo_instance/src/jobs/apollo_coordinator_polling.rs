use core::panic;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use apollo_utils::{
    address,
    errors::{MulticallError, Web3Error},
    get_metadata, get_state, log,
    multicall::{self, Call},
    nat::{ToNatType, ToNativeTypes},
    sybil::{self, get_sybil_input},
    update_state,
    web3::{self, Web3Instance},
};
use ic_web3_rs::{
    contract::{
        tokens::{Tokenizable, Tokenize},
        Contract, Options,
    },
    ethabi::Token,
    types::{H160, U256},
    Transport,
};

use crate::{
    types::{balances::Balances, ApolloCoordinatorRequest},
    utils::apollo_evm_address,
};
use ic_web3_rs::ethabi::Function;

const APOLLO_COORDINATOR_ABI: &[u8] =
    include_bytes!("../../../../assets/ApolloCoordinatorABI.json");
const TARGET_FUNCTION_ABI: &str = include_str!("../../../../assets/TargetFunctionABI.json");
const APOLLO_COORDINATOR_GET_REQUESTS_FROM_ID: &str = "getRequestsFromId";

async fn get_requests<T: Transport>(
    w3: &Web3Instance<T>,
    apollo_coordinator_contract: Contract<T>,
    from: String,
    gas_price: U256,
) -> Result<Vec<Token>> {
    log!(
        "[EXECUTION] get_requests: chain: {}, gas price: {}",
        get_metadata!(chain_id),
        gas_price,
    );

    let mut options = Options {
        gas_price: Some(gas_price),
        nonce: Some(w3.get_nonce(&apollo_evm_address().await?).await?),
        ..Default::default()
    };

    let request_from_id = if let Some(last_request_id) = get_state!(last_request_id) {
        last_request_id + 1
    } else {
        0
    };

    let params: Token = Token::Uint(request_from_id.into());

    log!(
        "[EXECUTION] get_requests: chain: {}, estimating gas",
        get_metadata!(chain_id)
    );

    log!("PARAMS: {:?}", params.clone());
    log!("FROM: {:?}", from.clone());

    let gas_limit = Web3Instance::estimate_gas(
        &apollo_coordinator_contract,
        APOLLO_COORDINATOR_GET_REQUESTS_FROM_ID,
        params.clone(),
        &from,
        &options,
    )
    .await
    .map_err(|err| Web3Error::UnableToEstimateGas(err.to_string()))?;

    log!(
        "[EXECUTION] get_requests: chain: {}, estimated_gas & limit: {}",
        get_metadata!(chain_id),
        gas_limit
    );

    options.gas = Some(gas_limit);

    let signed_call = w3
        .sign(
            &apollo_coordinator_contract,
            APOLLO_COORDINATOR_GET_REQUESTS_FROM_ID,
            vec![params.clone()],
            options.clone(),
            from.clone(),
            get_metadata!(key_name),
            get_metadata!(chain_id).to_u64(),
        )
        .await?;

    // TODO: Delete
    // without balance check rpc returns insufficient balance error (or not)
    log!(
        "BALANCE {}: {:?}",
        from,
        w3.get_address_balance(&from).await?
    );

    log!(
        "[EXECUTION] get_requests: chain: {}, sending tx",
        get_metadata!(chain_id)
    );

    let tx_receipt = w3.send_raw_transaction_and_wait(signed_call).await?;

    log!(
        "[EXECUTION] get_requests: chain: {}, tx was executed",
        get_metadata!(chain_id)
    );

    let call_result = w3
        .get_call_result(
            &apollo_coordinator_contract,
            APOLLO_COORDINATOR_GET_REQUESTS_FROM_ID,
            &[params],
            tx_receipt.from,
            tx_receipt.to,
            tx_receipt.block_number,
        )
        .await?
        .first()
        .cloned()
        .expect("Should have a result");

    let requests = call_result
        .into_array()
        .ok_or(anyhow!("Return type is not an array"))?;

    log!("[EXECUTION] get_requests: Returning requests");

    Ok(requests)
}

pub async fn update_last_request_id() -> Result<()> {
    let w3 = web3::instance(&get_metadata!(chain_rpc))?;
    let apollo_coordinator_address = address::to_h160(&get_metadata!(apollo_coordinator))?;
    let apollo_coordinator_contract =
        Contract::from_json(w3.eth(), apollo_coordinator_address, APOLLO_COORDINATOR_ABI)?;
    let from = apollo_evm_address().await?;

    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    let gas_price: U256 = (w3.get_gas_price().await? / 10) * 12;

    let call_result = get_requests(&w3, apollo_coordinator_contract, from, gas_price).await?;

    let call_result = call_result
        .last()
        .ok_or(anyhow!("No requests found"))?
        .clone();

    let last_request = ApolloCoordinatorRequest::from_token(call_result)?;

    update_state!(last_request_id, Some(last_request.request_id.as_u64()));

    Ok(())
}

pub async fn _execute() -> Result<()> {
    let w3 = web3::instance(&get_metadata!(chain_rpc))?;
    let apollo_coordinator_address = address::to_h160(&get_metadata!(apollo_coordinator))?;
    let apollo_coordinator_contract =
        Contract::from_json(w3.eth(), apollo_coordinator_address, APOLLO_COORDINATOR_ABI)?;
    let from = apollo_evm_address().await?;

    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    let gas_price: U256 = (w3.get_gas_price().await? / 10) * 12;

    let call_result = get_requests(&w3, apollo_coordinator_contract, from, gas_price).await?;

    process_requests(&w3, call_result, gas_price).await
}

async fn process_requests<T: Transport>(
    w3: &Web3Instance<T>,
    requests: Vec<Token>,
    gas_price: U256,
) -> Result<()> {
    if requests.is_empty() {
        log!("[EXECUTION] No requests found");
        return Ok(());
    }

    let mut last_request_id = 0.into();

    let mut calls = Vec::with_capacity(requests.len());

    for request in requests {
        let apollo_coordinator_request = ApolloCoordinatorRequest::from_token(request)?;

        // TODO: implement balance check
        // if Balances::get(&format!("{:?}", apollo_coordinator_request.requester))?.amount
        //     < get_metadata!(min_balance)
        // {
        //     log!(
        //         "[EXECUTION] chain: {}, not enough balance for requester {}",
        //         get_metadata!(chain_id),
        //         apollo_coordinator_request.requester
        //     );

        //     continue;
        // }

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

        last_request_id = apollo_coordinator_request.request_id;
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

        // TODO: implement balance reduction
        // let amount = gas_price.to_nat() * result.used_gas.to_nat() + get_metadata!(apollos_fee);

        // Balances::reduce_amount(&format!("{:?}", call.target), &amount)
        //     .expect("should reduce balance");
        // TODO: add fee collection
    }

    update_state!(last_request_id, Some(last_request_id.as_u64()));
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
