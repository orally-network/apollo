use apollo_utils::sybil::SybilAssetData;
use candid::CandidType;
use ic_cdk::api::management_canister::main::raw_rand;
use ic_web3_rs::{ethabi::Token, types::U256};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, CandidType, Deserialize, Serialize)]
pub enum AssetData {
    DefaultPriceFeed {
        request_id: u64,
        symbol: String,
        rate: u64,
        decimals: u64,
        timestamp: u64,
    },
    CustomPriceFeed {
        request_id: u64,
        symbol: String,
        rate: u64,
        decimals: u64,
        timestamp: u64,
    },
    CustomNumber {
        request_id: u64,
        id: String,
        value: u64,
        decimals: u64,
    },
    CustomString {
        request_id: u64,
        id: String,
        value: String,
    },
    Random {
        request_id: u64,
        num_words: u64,
    },
}

impl AssetData {
    pub async fn encode(&self) -> Vec<Token> {
        match self.clone() {
            AssetData::DefaultPriceFeed {
                request_id,
                symbol,
                rate,
                decimals,
                timestamp,
            } => vec![
                Token::Uint(request_id.into()),
                Token::String(symbol.clone()),
                Token::Uint(rate.into()),
                Token::Uint(decimals.into()),
                Token::Uint(timestamp.into()),
            ],
            AssetData::CustomPriceFeed {
                request_id,
                symbol,
                rate,
                decimals,
                timestamp,
            } => {
                vec![
                    Token::Uint(request_id.into()),
                    Token::String(symbol.clone()),
                    Token::Uint(rate.into()),
                    Token::Uint(decimals.into()),
                    Token::Uint(timestamp.into()),
                ]
            }
            AssetData::CustomNumber {
                request_id,
                id,
                value,
                decimals,
            } => {
                vec![
                    Token::Uint(request_id.into()),
                    Token::String(id.clone()),
                    Token::Uint(value.into()),
                    Token::Uint(decimals.into()),
                ]
            }
            AssetData::CustomString {
                request_id,
                id,
                value,
            } => {
                vec![
                    Token::Uint(request_id.into()),
                    Token::String(id.clone()),
                    Token::String(value.clone()),
                ]
            }
            AssetData::Random {
                request_id,
                num_words: num_of_random_bytes,
            } => {
                let mut vec = Vec::with_capacity(num_of_random_bytes as usize);
                for _ in 0..num_of_random_bytes {
                    let (random,) = raw_rand().await.expect("should be able to get random");
                    vec.push(Token::Uint(U256::from_big_endian(&random)));
                }

                vec![Token::Uint(request_id.into()), Token::Array(vec)]
            }
        }
    }

    pub fn from_sybil_asset_data_and_req_id(req_id: u64, sybil_asset_data: SybilAssetData) -> Self {
        match sybil_asset_data {
            SybilAssetData::DefaultPriceFeed {
                symbol,
                rate,
                decimals,
                timestamp,
            } => AssetData::DefaultPriceFeed {
                request_id: req_id,
                symbol,
                rate,
                decimals,
                timestamp,
            },
            SybilAssetData::CustomPriceFeed {
                symbol,
                rate,
                decimals,
                timestamp,
            } => AssetData::CustomPriceFeed {
                request_id: req_id,
                symbol,
                rate,
                decimals,
                timestamp,
            },
            SybilAssetData::CustomNumber {
                id,
                value,
                decimals,
            } => AssetData::CustomNumber {
                request_id: req_id,
                id,
                value,
                decimals,
            },
            SybilAssetData::CustomString { id, value } => AssetData::CustomString {
                request_id: req_id,
                id,
                value,
            },
        }
    }
}
