use anyhow::Result;
use candid::Principal;
use ic_cdk::api::management_canister::http_request::{TransformContext, TransformFunc};
use ic_web3_rs::{
    api::Eth,
    contract::{
        tokens::{Tokenizable, Tokenize},
        Contract, Options,
    },
    ethabi::Token,
    ic::KeyInfo,
    transports::ic_http_client::CallOptionsBuilder,
    types::{
        BlockId, BlockNumber, Bytes, CallRequest, FilterBuilder, Log, SignedTransaction,
        Transaction, TransactionId, TransactionReceipt, H160, H256, U256, U64,
    },
    Transport, Web3,
};
use std::{str::FromStr, time::Duration};

use crate::{
    errors::{UtilsError, Web3Error},
    http, log, retry_until_success, time,
};

use self::evm_canister_transport::EVMCanisterTransport;

const ECDSA_SIGN_CYCLES: u64 = 23_000_000_000;
pub const TRANSFER_GAS_LIMIT: u64 = 21_000;
const TX_SUCCESS_STATUS: u64 = 1;
const TX_WAIT_DELAY: Duration = Duration::from_secs(3);
const TX_WAITING_TIMEOUT: u64 = 60 * 5;

mod evm_canister_transport;

pub struct Web3Instance<T: Transport> {
    w3: Web3<T>,
}

pub fn instance(
    rpc_url: String,
    evm_rpc_canister: String,
) -> Result<Web3Instance<impl Transport>, Web3Error> {
    // Switch between EVMCanisterTransport(calls go through emv_rpc canister) and ICHttp (calls go straight to the rpc)

    Ok(Web3Instance::new(Web3::new(EVMCanisterTransport::new(
        rpc_url,
        Principal::from_str(&evm_rpc_canister).expect("should be a valid principal"),
    ))))

    // Ok(Web3Instance::new(Web3::new(
    //     ICHttp::new(&rpc_url, None).unwrap(),
    // )))
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

    pub async fn get_block_number(&self) -> Result<u64, Web3Error> {
        self.eth()
            .block_number(http::transform_ctx())
            .await
            .map_err(|err| Web3Error::UnableToGetBlockNumber(err.to_string()))
            .map(|val| val.as_u64())
    }

    pub async fn get_logs(
        &self,
        from: u64,
        to: Option<u64>,
        topic: Option<H256>,
        address: Option<H160>,
    ) -> Result<Vec<Log>, Web3Error> {
        let filter_builder = FilterBuilder::default();
        let to_block = if let Some(to) = to {
            BlockNumber::Number(to.into())
        } else {
            BlockNumber::Latest
        };

        let topic1 = if let Some(topic) = topic {
            Some(vec![topic])
        } else {
            None
        };

        let address = if let Some(address) = address {
            vec![address]
        } else {
            vec![]
        };

        let logs = self
            .eth()
            .logs(
                filter_builder
                    .from_block(BlockNumber::Number(from.into()))
                    .to_block(to_block)
                    .topics(topic1, None, None, None)
                    .address(address)
                    .build(),
                http::transform_ctx(),
            )
            .await
            .map_err(|err| Web3Error::UnableToGetLogs(err.to_string()))?;

        Ok(logs)
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

    pub async fn get_tx(&self, tx_hash: &str) -> Result<Transaction, Web3Error> {
        let tx_hash =
            H256::from_str(tx_hash).map_err(|err| UtilsError::FromHexError(err.to_string()))?;

        let tx_receipt = retry_until_success!(self
            .eth()
            .transaction_receipt(tx_hash, http::transform_ctx_tx_with_logs()))
        .map_err(|err| Web3Error::UnableToGetTxReceipt(err.to_string()))?
        .ok_or(Web3Error::TxNotFound)?;

        match tx_receipt.status {
            Some(status) => {
                if status.as_u64() != 1 {
                    return Err(Web3Error::TxHasFailed);
                }
            }
            None => return Err(Web3Error::TxNotFound),
        }

        let result = retry_until_success!(self
            .eth()
            .transaction(TransactionId::from(tx_hash), http::transform_ctx_tx()))
        .map_err(|err| Web3Error::UnableToGetTxReceipt(err.to_string()))?
        .ok_or(Web3Error::TxNotFound)?;

        Ok(result)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn sign<Tk: Tokenizable + Clone>(
        &self,
        contract: &Contract<T>,
        func: &str,
        params: Vec<Tk>,
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

    // We need to pass custom wrapper around args because ic-web3-rs doesn't parse them as
    // vec of tokens, but as a single token
    pub async fn estimate_gas<P: Tokenize + Clone>(
        contract: &Contract<T>,
        func: &str,
        params: P,
        from: &str,
        options: &Options,
    ) -> Result<U256, Web3Error> {
        let estimated_gas = retry_until_success!(contract.estimate_gas(
            func,
            params.clone(),
            H160::from_str(from).map_err(|err| Web3Error::InvalidAddressFormat(err.to_string()))?,
            options.clone(),
        ))
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
        log!("[EXECUTION] Getting call result");
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

        log!("[EXECUTION] Getting raw result");

        let raw_result = retry_until_success!(self.eth().call(
            call_request.clone(),
            block_number,
            http::transform_ctx()
        ))
        .map_err(|err| Web3Error::UnableToCallContract(err.to_string()))?;

        log!("[EXECUTION] Converting raw result");

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

    pub async fn send_raw_transaction(
        &self,
        signed_call: SignedTransaction,
    ) -> Result<H256, Web3Error> {
        let tx_hash = retry_until_success!(self
            .eth()
            .send_raw_transaction(signed_call.raw_transaction.clone(), http::transform_ctx()))
        .map_err(|err| Web3Error::UnableToExecuteRawTx(err.to_string()))?;

        Ok(tx_hash)
    }

    pub async fn send_raw_transaction_and_wait(
        &self,
        signed_call: SignedTransaction,
    ) -> Result<TransactionReceipt, Web3Error> {
        let tx_hash = self.send_raw_transaction(signed_call).await?;

        self.wait_for_success_confirmation(tx_hash).await
    }

    pub async fn wait_for_success_confirmation(
        &self,
        tx_hash: H256,
    ) -> Result<TransactionReceipt, Web3Error> {
        let receipt = self.wait_for_confirmation(&tx_hash).await?;

        let tx_status = receipt.status.expect("tx should be confirmed").as_u64();

        if tx_status != TX_SUCCESS_STATUS {
            log!("ERRONEOUS TX LOGS: {:?}", receipt.logs);
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
