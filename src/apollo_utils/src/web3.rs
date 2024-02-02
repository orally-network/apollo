use anyhow::Result;
use ic_cdk::api::management_canister::http_request::{TransformContext, TransformFunc};
use ic_web3_rs::{
    api::Eth,
    contract::{Contract, Options},
    ethabi::Token,
    ic::KeyInfo,
    transports::{ic_http_client::CallOptionsBuilder, ICHttp},
    types::{
        BlockId, Bytes, CallRequest, SignedTransaction, TransactionReceipt, H160, H256, U256, U64,
    },
    Transport, Web3,
};
use std::{str::FromStr, time::Duration};

use crate::{errors::Web3Error, http, retry_until_success, time};

const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;
pub const TRANSFER_GAS_LIMIT: u64 = 21_000;
const TX_SUCCESS_STATUS: u64 = 1;
const TX_WAIT_DELAY: Duration = Duration::from_secs(3);
const TX_WAITING_TIMEOUT: u64 = 60 * 5;

pub struct Web3Instance<T: Transport> {
    w3: Web3<T>,
}

pub fn instance(rpc: &str) -> Result<Web3Instance<ICHttp>, Web3Error> {
    Ok(Web3Instance::new(Web3::new(
        ICHttp::new(rpc, None).expect("should be able to create http transport"),
    )))
}

impl<T: Transport> Web3Instance<T> {
    pub fn new(w3: Web3<T>) -> Self {
        Self { w3 }
    }

    pub fn eth(&self) -> Eth<T> {
        self.w3.eth()
    }

    #[inline(always)]
    pub fn key_info(key_name: String) -> KeyInfo {
        KeyInfo {
            derivation_path: vec![ic_cdk::id().as_slice().to_vec()],
            key_name,
            ecdsa_sign_cycles: Some(ECDSA_SIGN_CYCLES),
        }
    }

    pub async fn get_address_balance(&self, address: &str) -> Result<U256, Web3Error> {
        let balance = retry_until_success!(self.eth().balance(
            H160::from_str(address)
                .map_err(|err| Web3Error::InvalidAddressFormat(err.to_string()))?,
            None,
            http::transform_ctx()
        ))
        .map_err(|err| Web3Error::UnableToEstimateGas(err.to_string()))?;

        Ok(balance)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn sign(
        &self,
        contract: &Contract<T>,
        func: &str,
        params: Vec<Token>,
        options: Options,
        from: String,
        key_name: String,
        chain_id: u64,
    ) -> Result<SignedTransaction, Web3Error> {
        let signed_call = contract
            .sign(
                func,
                params,
                options,
                from,
                Self::key_info(key_name),
                chain_id,
            )
            .await
            .map_err(|err| Web3Error::UnableToSignContractCall(err.to_string()))?;

        Ok(signed_call)
    }

    pub async fn estimate_gas(
        contract: &Contract<T>,
        func: &str,
        params: &Vec<Token>,
        from: &str,
        options: &Options,
    ) -> Result<U256, Web3Error> {
        let estimated_gas = contract
            .estimate_gas(
                func,
                params.clone(),
                H160::from_str(from)
                    .map_err(|err| Web3Error::InvalidAddressFormat(err.to_string()))?,
                options.clone(),
            )
            .await
            .map_err(|err| Web3Error::UnableToEstimateGas(err.to_string()))?;

        Ok(estimated_gas)
    }

    pub async fn get_call_result(
        &self,
        contract: &Contract<T>,
        func: &str,
        params: &[Token],
        from: H160,
        to: Option<H160>,
        block_number: Option<U64>,
    ) -> Result<Vec<Token>, Web3Error> {
        let data = contract
            .abi()
            .function(func)
            .and_then(|f| f.encode_input(params))
            .map_err(|err| Web3Error::UnableToFormCallData(err.to_string()))?;

        let call_request = CallRequest {
            from: Some(from),
            to,
            data: Some(Bytes::from(data)),
            ..Default::default()
        };

        let block_number = block_number.map(|block_number| BlockId::Number(block_number.into()));

        let raw_result = retry_until_success!(self.eth().call(
            call_request.clone(),
            block_number,
            http::transform_ctx()
        ))
        .map_err(|err| Web3Error::UnableToCallContract(err.to_string()))?;

        let call_result: Vec<Token> = contract
            .abi()
            .function(func)
            .and_then(|f| f.decode_output(&raw_result.0))
            .map_err(|err| Web3Error::UnableToDecodeOutput(err.to_string()))?;

        Ok(call_result)
    }

    pub async fn get_gas_price(&self) -> Result<U256, Web3Error> {
        let gas_price = match retry_until_success!(self.eth().gas_price(http::transform_ctx())) {
            Ok(gas_price) => gas_price,
            Err(e) => Err(Web3Error::UnableToGetGasPrice(e.to_string()))?,
        };

        Ok(gas_price)
    }

    pub async fn get_nonce(&self, account_address: &str) -> Result<U256, Web3Error> {
        let nonce = match retry_until_success!(self.eth().transaction_count(
            H160::from_str(account_address)
                .map_err(|err| Web3Error::InvalidAddressFormat(err.to_string()))?,
            None,
            http::transform_ctx()
        )) {
            Ok(nonce) => nonce,
            Err(e) => Err(Web3Error::UnableToGetNonce(e.to_string()))?,
        };

        Ok(nonce)
    }

    pub async fn send_raw_transaction_and_wait(
        &self,
        signed_call: SignedTransaction,
    ) -> Result<TransactionReceipt, Web3Error> {
        let tx_hash = retry_until_success!(self
            .eth()
            .send_raw_transaction(signed_call.raw_transaction.clone(), http::transform_ctx()))
        .map_err(|err| Web3Error::UnableToExecuteRawTx(err.to_string()))?;

        self.wait_for_success_confirmation(tx_hash).await
    }

    pub async fn wait_for_success_confirmation(
        &self,
        tx_hash: H256,
    ) -> Result<TransactionReceipt, Web3Error> {
        let receipt = self.wait_for_confirmation(&tx_hash).await?;

        let tx_status = receipt.status.expect("tx should be confirmed").as_u64();

        if tx_status != TX_SUCCESS_STATUS {
            return Err(Web3Error::TxHasFailed);
        }

        Ok(receipt)
    }

    pub async fn wait_for_confirmation(
        &self,
        tx_hash: &H256,
    ) -> Result<TransactionReceipt, Web3Error> {
        let call_opts = CallOptionsBuilder::default()
            .transform(Some(TransformContext {
                function: TransformFunc(candid::Func {
                    principal: ic_cdk::api::id(),
                    method: "transform".into(),
                }),
                context: vec![],
            }))
            .cycles(None)
            .max_resp(None)
            .build()
            .expect("failed to build call options");

        let end_time = time::in_seconds() + TX_WAITING_TIMEOUT;
        while time::in_seconds() < end_time {
            time::sleep(TX_WAIT_DELAY).await;

            let tx_receipt =
                retry_until_success!(self.eth().transaction_receipt(*tx_hash, call_opts.clone()))
                    .map_err(|err| Web3Error::UnableToGetTxReceipt(err.to_string()))?;

            if let Some(tx_receipt) = tx_receipt {
                if tx_receipt.status.is_some() {
                    return Ok(tx_receipt);
                }
            }
        }

        Err(Web3Error::TxTimeout)
    }
}
