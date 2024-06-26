use std::str::FromStr;

use apollo_utils::{
    address,
    errors::LogsPoolingError,
    get_metadata, get_state, log, update_state,
    web3::{self, Web3Instance},
};
use ic_web3_rs::{
    ethabi::{self, RawLog},
    types::{H256, U256},
    Transport,
};

use crate::types::ApolloCoordinatorRequest;

use super::process_requests;

const APOLLO_COORDINATOR_ABI: &[u8] =
    include_bytes!("../../../../assets/ApolloCoordinatorABI.json");
const APOOLLO_COORDINATOR_DATA_FEED_REQUESTED_EVENT_NAME: &str = "DataFeedRequested";
const APOOLLO_COORDINATOR_RANDOM_FEED_REQUESTED_EVENT_NAME: &str = "RandomFeedRequested";
const DATA_FEED_REQUESTED_TOPIC: &str =
    "0x23752ae5400f82f705b104bd992d5ae9631719e025bb5934d3ed82d5aa9c27ee";
const RANDOM_FEED_REQUESTED_TOPIC: &str =
    "0x266a11fde650ce93d717e570d9ebbfa8746356fe5c7c73647a03d56dde7027c0";

pub async fn _execute() -> Result<(), LogsPoolingError> {
    let w3 = web3::instance(get_metadata!(chain_rpc), get_metadata!(evm_rpc_canister))?;

    let (requests, last_block) = get_requests(&w3).await?;

    // multiply the gas_price to 1.2 to avoid long transaction confirmation
    let gas_price: U256 = (w3.get_gas_price().await? * 12) / 10;

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

    let logs_result = w3
        .get_logs(
            last_parsed_logs_from_block,
            None,
            Some(vec![
                H256::from_str(DATA_FEED_REQUESTED_TOPIC).expect("should be able to parse"),
                H256::from_str(RANDOM_FEED_REQUESTED_TOPIC).expect("should be able to parse"),
            ]),
            Some(address::to_h160(&get_metadata!(apollo_coordinator))?),
        )
        .await;

    log!("Logs result: {:?}", logs_result);

    let logs = match logs_result {
        Ok(logs) => logs,
        Err(err) => {
            if err.to_string().contains("block range is too wide") {
                log!("[EXECUTION] Block range is too wide, updating last_parsed_logs_from_block to the latest block");
                return Ok((vec![], w3.get_block_number().await?));
            }

            return Err(err.into());
        }
    };

    let apollo_coordinator_abi = ethabi::Contract::load(APOLLO_COORDINATOR_ABI).unwrap();

    let data_feed_event = apollo_coordinator_abi
        .event(APOOLLO_COORDINATOR_DATA_FEED_REQUESTED_EVENT_NAME)
        .expect("should be able to get event by name");
    let random_feed_event = apollo_coordinator_abi
        .event(APOOLLO_COORDINATOR_RANDOM_FEED_REQUESTED_EVENT_NAME)
        .expect("should be able to get event by name");

    let mut requests = Vec::with_capacity(logs.len());

    if logs.is_empty() {
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

        let first_topic = format!(
            "{:?}",
            raw_log
                .topics
                .first()
                .expect("should be able to get first topic")
        );

        log!("First topic: {:?}", first_topic);

        match first_topic.as_str() {
            DATA_FEED_REQUESTED_TOPIC => {
                let parsed_log = data_feed_event
                    .parse_log(raw_log)
                    .map_err(|err| LogsPoolingError::AbiParsingError(err.to_string()))?;

                requests.push(ApolloCoordinatorRequest::new_from_data_feed_log(parsed_log))
            }
            RANDOM_FEED_REQUESTED_TOPIC => {
                let parsed_log = random_feed_event
                    .parse_log(raw_log)
                    .map_err(|err| LogsPoolingError::AbiParsingError(err.to_string()))?;

                requests.push(ApolloCoordinatorRequest::new_from_random_feed_log(
                    parsed_log,
                ))
            }
            _ => unreachable!("should be able to parse"),
        }
    }

    log!("[EXECUTION] Found {} requests", requests.len());

    Ok((requests, last_parsed_block))
}
