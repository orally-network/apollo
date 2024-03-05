use anyhow::{anyhow, Result};
use candid::{CandidType, Principal};
use ic_web3_rs::ethabi::Token;
use serde::{Deserialize, Serialize};

use crate::{errors::SybilError, log, retry_until_success};

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
enum FeedDataResponse {
    Ok(AssetDataResult),
    Err(String),
}

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum AssetData {
    DefaultPriceFeed {
        symbol: String,
        rate: u64,
        decimals: u64,
        timestamp: u64,
    },
    CustomPriceFeed {
        symbol: String,
        rate: u64,
        decimals: u64,
        timestamp: u64,
    },
    CustomNumber {
        id: String,
        value: u64,
        decimals: u64,
    },
    CustomString {
        id: String,
        value: String,
    },
}

impl Default for AssetData {
    fn default() -> Self {
        AssetData::DefaultPriceFeed {
            symbol: "".to_string(),
            rate: 0,
            decimals: 0,
            timestamp: 0,
        }
    }
}

/// Result of the asset data request from sybil
/// Copy-pasted from sybil in order to avoid lots of imports
#[derive(Clone, Default, Debug, CandidType, Serialize, Deserialize)]
pub struct AssetDataResult {
    pub data: AssetData,
    pub signature: Option<String>,
}

impl AssetData {
    pub fn encode(&self) -> Vec<Token> {
        match self.clone() {
            AssetData::DefaultPriceFeed {
                symbol,
                rate,
                decimals,
                timestamp,
            } => vec![
                Token::String(symbol.clone()),
                Token::Uint(rate.into()),
                Token::Uint(decimals.into()),
                Token::Uint(timestamp.into()),
            ],
            AssetData::CustomPriceFeed {
                symbol,
                rate,
                decimals,
                timestamp,
            } => {
                vec![
                    Token::String(symbol.clone()),
                    Token::Uint(rate.into()),
                    Token::Uint(decimals.into()),
                    Token::Uint(timestamp.into()),
                ]
            }
            AssetData::CustomNumber {
                id,
                value,
                decimals,
            } => {
                vec![
                    Token::String(id.clone()),
                    Token::Uint(value.into()),
                    Token::Uint(decimals.into()),
                ]
            }
            AssetData::CustomString { id, value } => {
                vec![Token::String(id.clone()), Token::String(value.clone())]
            }
        }
    }
}

pub async fn is_feed_exists(sybil_canister_address: Principal, feed_id: String) -> Result<bool> {
    log!("[SYBIL] Feed exists requested for feed_id: {:#?}", feed_id);

    let (is_exist,): (bool,) = retry_until_success!(ic_cdk::call(
        sybil_canister_address,
        "is_feed_exists",
        (feed_id.clone(),)
    ))
    .map_err(|(code, msg)| anyhow!("{:?}: {}", code, msg))?;

    log!(
        "[SYBIL] Feed exists for feed_id {:#?} returned {}",
        feed_id,
        is_exist
    );

    Ok(is_exist)
}

pub async fn get_asset_data(
    sybil_canister_address: Principal,
    feed_id: String,
) -> Result<AssetDataResult, SybilError> {
    log!("[SYBIL] Asset data requested for feed_id: {:#?}", feed_id);

    let (feed_data,): (Result<AssetDataResult, String>,) = retry_until_success!(ic_cdk::call(
        sybil_canister_address,
        "get_asset_data",
        (feed_id.clone(),)
    ))
    .map_err(|(_, msg)| SybilError::CanisterError(msg))?;

    log!("[SYBIL] Asset data returned");

    match feed_data {
        Result::Ok(data) => Ok(data),
        Result::Err(err) => Err(SybilError::CanisterError(err)),
    }
}

pub async fn get_sybil_feed(
    sybil_canister_address: String,
    feed_id: String,
) -> Result<AssetData, SybilError> {
    let sybil_canister_address = Principal::from_text(sybil_canister_address)
        .map_err(|err| SybilError::InvalidPrincipal(err.to_string()))?;

    log!(
        "[ABI] get_sybil_input requested sybil::get_asset_data, feed_id: {}",
        feed_id
    );

    let asset_data = retry_until_success!(get_asset_data(sybil_canister_address, feed_id.clone()))?;

    log!("[ABI] get_sybil_input got asset_data feed_id: {}", feed_id);
    Ok(asset_data.data)
}
