use anyhow::{anyhow, Result};
use apollo_utils::{
    address,
    errors::Web3Error,
    get_metadata, get_state, log,
    nat::ToNativeTypes,
    update_state,
    web3::{self, Web3Instance},
};
use ic_web3_rs::{
    contract::{tokens::Tokenizable, Contract, Options},
    ethabi::Token,
    types::U256,
    Transport,
};

use crate::{types::ApolloCoordinatorRequest, utils::apollo_evm_address};

use super::process_requests;

const APOLLO_COORDINATOR_ABI: &[u8] =
    include_bytes!("../../../../assets/ApolloCoordinatorABI.json");
const APOLLO_COORDINATOR_GET_REQUESTS_FROM_ID: &str = "getRequestsFromId";

pub async fn _execute() -> Result<()> {
    let w3 = web3::instance(&get_metadata!(chain_rpc))?;
    let apollo_coordinator_address = address::to_h160(&get_metadata!(apollo_coordinator))?;
    let apollo_coordinator_contract =
        Contract::from_json(w3.eth(), apollo_coordinator_address, APOLLO_COORDINATOR_ABI)?;
    let from = apollo_evm_address().await?;

    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    let gas_price: U256 = (w3.get_gas_price().await? / 10) * 12;

    let (requests, last_request_id) =
        get_requests(&w3, apollo_coordinator_contract, from, gas_price).await?;

    process_requests(&w3, requests, gas_price).await?;

    update_state!(last_request_id, Some(last_request_id));

    Ok(())
}

/// Returns the requests for the Apollo Coordinator contract
/// alongside with the last requestId
async fn get_requests<T: Transport>(
    w3: &Web3Instance<T>,
    apollo_coordinator_contract: Contract<T>,
    from: String,
    gas_price: U256,
) -> Result<(Vec<ApolloCoordinatorRequest>, u64)> {
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
    // log!(
    //     "BALANCE {}: {:?}",
    //     from,
    //     w3.get_address_balance(&from).await?
    // );

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

    let results = call_result
        .into_array()
        .ok_or(anyhow!("Return type is not an array"))?;

    let mut requests = Vec::with_capacity(results.len());

    for request in results {
        requests.push(ApolloCoordinatorRequest::from_token(request)?);
    }

    let last_request_id = requests.last().unwrap().request_id.as_u64();

    log!("[EXECUTION] get_requests: Returning requests");

    Ok((requests, last_request_id))
}

pub async fn update_last_request_id() -> Result<()> {
    let w3 = web3::instance(&get_metadata!(chain_rpc))?;
    let apollo_coordinator_address = address::to_h160(&get_metadata!(apollo_coordinator))?;
    let apollo_coordinator_contract =
        Contract::from_json(w3.eth(), apollo_coordinator_address, APOLLO_COORDINATOR_ABI)?;
    let from = apollo_evm_address().await?;

    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    let gas_price: U256 = (w3.get_gas_price().await? / 10) * 12;

    let (_, last_request_id) =
        get_requests(&w3, apollo_coordinator_contract, from, gas_price).await?;

    update_state!(last_request_id, Some(last_request_id));

    Ok(())
}
