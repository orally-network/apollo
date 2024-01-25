use anyhow::Result;
use apollo_utils::{address, errors::Web3Error, get_metadata, log, nat::ToNativeTypes, web3};
use ic_web3_rs::{
    contract::{Contract, Options},
    ethabi::Token,
    types::U256,
};

use crate::utils::apollo_evm_address;

const APOLLO_COORDINATOR_ABI: &[u8] = include_bytes!("../../../../ApolloCoordinatorABI.json");
const APOLLO_COORDINATOR_GET_REQUESTS_FROM_ID: &str = "getRequestsFromId";

pub async fn _execute() -> Result<()> {
    let w3 = web3::instance(&get_metadata!(chain_rpc))?;
    let apollo_coordinator_address = address::to_h160(&get_metadata!(apollo_coordinator))?;
    let apollo_coordinator_contract =
        Contract::from_json(w3.eth(), apollo_coordinator_address, APOLLO_COORDINATOR_ABI)?;
    let from = address::to_h160(&apollo_evm_address().await?)?;

    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    let gas_price: U256 = (w3.get_gas_price().await? / 10) * 12;

    log!(
        "[EXECUTION] chain: {}, gas price: {}",
        get_metadata!(chain_id),
        gas_price,
    );

    let mut options = Options {
        gas_price: Some(gas_price),
        nonce: Some(w3.get_nonce(&apollo_evm_address().await?).await?),
        ..Default::default()
    };

    let params: Token = Token::Uint(0.into()); // TODO: change into proper value

    let gas_limit = apollo_coordinator_contract
        .estimate_gas(
            APOLLO_COORDINATOR_GET_REQUESTS_FROM_ID,
            params.clone(),
            from,
            options.clone(),
        )
        .await
        .map_err(|err| Web3Error::UnableToEstimateGas(err.to_string()))?;

    options.gas = Some(gas_limit);

    let signed_call = w3
        .sign(
            &apollo_coordinator_contract,
            APOLLO_COORDINATOR_GET_REQUESTS_FROM_ID,
            vec![params.clone()],
            options.clone(),
            from.to_string(),
            get_metadata!(key_name),
            get_metadata!(chain_id).to_u64(),
        )
        .await?;

    log!("[EXECUTION] chain: {}, sending tx", get_metadata!(chain_id));

    let tx_receipt = w3.send_raw_transaction_and_wait(signed_call).await?;

    log!(
        "[EXECUTION] chain: {}, tx was executed",
        get_metadata!(chain_id)
    );

    let call_result = w3
        .get_call_result(
            &apollo_coordinator_contract,
            APOLLO_COORDINATOR_GET_REQUESTS_FROM_ID,
            &[params],
            tx_receipt,
        )
        .await?;

    log!("RESULT: {:#?}", call_result);

    Ok(())
}
