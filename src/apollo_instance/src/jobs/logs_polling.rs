use std::str::FromStr;

use apollo_utils::{
    address,
    errors::LogsPoolingError,
    get_metadata, get_state, log, update_state,
    web3::{self, Web3Instance},
};
use ic_web3_rs::{
    ethabi::{Event, EventParam, ParamType, RawLog},
    types::{H256, U256},
    Transport,
};

use crate::types::ApolloCoordinatorRequest;

use super::process_requests;

const PRICE_FEED_REQUESTED_TOPIC: &str =
    "0x5fa8d8d6d4d7f09f9e3aac2baeb1e84cb5b9974d628ff44a7c91297cd2c65025";

pub async fn _execute() -> Result<(), LogsPoolingError> {
    let w3 = web3::instance(&get_metadata!(chain_rpc))?;

    let (requests, last_block) = get_requests(&w3).await?;

    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    let gas_price: U256 = (w3.get_gas_price().await? / 10) * 12;

    log!("Got requests: {:?}", requests);

    process_requests(&w3, requests, gas_price)
        .await
        .map_err(|err| LogsPoolingError::FailedToProcessRequests(err.to_string()))?;

    update_state!(last_parsed_logs_from_block, Some(last_block));

    Ok(())
}

/// Returns the requests for the Apollo Coordinator contract
/// alongside the last block number from which the logs were parsed
async fn get_requests<T: Transport>(
    w3: &Web3Instance<T>,
) -> Result<(Vec<ApolloCoordinatorRequest>, u64), LogsPoolingError> {
    let last_parsed_logs_from_block =
        if let Some(last_parsed) = get_state!(last_parsed_logs_from_block) {
            last_parsed + 1
        } else {
            let last_block = w3.get_block_number().await?;
            update_state!(last_parsed_logs_from_block, Some(last_block));
            last_block
        };

    log!(
        "[EXECUTION] Getting logs from block {} to the latest block",
        last_parsed_logs_from_block
    );

    let logs = w3
        .get_logs(
            // 5273204,
            last_parsed_logs_from_block,
            // Some(5273204),
            None,
            Some(H256::from_str(PRICE_FEED_REQUESTED_TOPIC).expect("should be able to parse")),
            Some(address::to_h160(&get_metadata!(apollo_coordinator))?),
        )
        .await?;

    let abi_event = get_abi_event();

    let mut requests = Vec::with_capacity(logs.len());

    if logs.is_empty() {
        log!("[EXECUTION] No logs found");
        return Ok((requests, last_parsed_logs_from_block));
    }

    let last_parsed_block = logs
        .last()
        .map(|log| log.block_number)
        .unwrap()
        .expect("should be able to get last block number")
        .as_u64();

    for log in logs {
        let raw_log = RawLog {
            topics: log.topics,
            data: log.data.0,
        };

        let parsed_log = abi_event
            .parse_log(raw_log)
            .map_err(|err| LogsPoolingError::AbiParsingError(err.to_string()))?;

        requests.push(ApolloCoordinatorRequest::from(parsed_log));
    }

    log!("[EXECUTION] Found {} requests", requests.len());

    Ok((requests, last_parsed_block))
}

fn get_abi_event() -> Event {
    let params = vec![
        EventParam {
            name: "requestId".to_string(),
            kind: ParamType::Uint(256),
            indexed: true,
        },
        EventParam {
            name: "dataFeedId".to_string(),
            kind: ParamType::String,
            indexed: false,
        },
        EventParam {
            name: "callbackGasLimit".to_string(),
            kind: ParamType::Uint(256),
            indexed: false,
        },
        EventParam {
            name: "requester".to_string(),
            kind: ParamType::Address,
            indexed: true,
        },
    ];

    Event {
        name: "PriceFeedRequested".to_string(),
        inputs: params,
        anonymous: false,
    }
}
