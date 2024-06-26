use std::str::FromStr;

use ic_web3_rs::{
    contract::{tokens::Tokenizable, Contract, Error, Options},
    ethabi::{self, Event, RawLog, Token},
    types::{H160, H256, U256},
    Transport,
};

use crate::{
    address,
    errors::{MulticallError, Web3Error},
    log,
    web3::Web3Instance,
};

const MULTICALL_ABI: &[u8] = include_bytes!("../../../assets/MulticallABI.json");
const MULTICALL_CALL_FUNCTION: &str = "multicall";
const MULTICALL_TRANSFER_FUNCTION: &str = "multitransfer";
const MULTICALL_EXECUTED_TOPIC: &str =
    "0xf8bdf3986d2670f09fe23c388ce864efb28f948b773dcf51e3839f6262c2cb4f";
const MULTICALL_EXECUTED_EVENT_NAME: &str = "MulticallExecuted";
pub const BASE_GAS: u64 = 27_000;
const GAS_FOR_OPS: u64 = 10_000;
pub const GAS_PER_TRANSFER: u64 = 7_900;

#[derive(Debug, Clone, Default)]
pub struct Call {
    pub target: H160,
    pub call_data: Vec<u8>,
    pub gas_limit: U256,
}

impl Tokenizable for Call {
    fn from_token(token: Token) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Token::Tuple(tokens) = token {
            if tokens.len() != 3 {
                return Err(Error::InvalidOutputType("invalid tokens number".into()));
            }

            if let (Token::Address(target), Token::Bytes(call_data), Token::Uint(gas_limit)) =
                (tokens[0].clone(), tokens[1].clone(), tokens[2].clone())
            {
                return Ok(Self {
                    target,
                    call_data,
                    gas_limit,
                });
            }
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            Token::Address(self.target),
            Token::Bytes(self.call_data),
            Token::Uint(self.gas_limit),
        ])
    }
}

#[derive(Debug, Clone, Default)]
pub struct MulticallResult {
    pub success: bool,
    pub used_gas: U256,
    pub return_data: Vec<u8>,
}

impl Tokenizable for MulticallResult {
    fn from_token(token: Token) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Token::Tuple(tokens) = token {
            if tokens.len() != 3 {
                return Err(Error::InvalidOutputType("invalid tokens number".into()));
            }

            if let (Token::Bool(success), Token::Uint(used_gas), Token::Bytes(return_data)) =
                (tokens[0].clone(), tokens[1].clone(), tokens[2].clone())
            {
                return Ok(Self {
                    success,
                    used_gas,
                    return_data,
                });
            }
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            Token::Bool(self.success),
            Token::Bytes(self.return_data),
        ])
    }
}

#[derive(Debug, Clone, Default)]
pub struct Transfer {
    pub target: H160,
    pub value: U256,
    pub from: String,
}

impl Tokenizable for Transfer {
    fn from_token(token: Token) -> std::result::Result<Self, Error>
    where
        Self: Sized,
    {
        if let Token::Tuple(tokens) = token {
            if tokens.len() != 2 {
                return Err(Error::InvalidOutputType("invalid tokens number".into()));
            }

            if let (Token::Address(target), Token::Uint(value)) =
                (tokens[0].clone(), tokens[1].clone())
            {
                return Ok(Self {
                    target,
                    value,
                    from: "".into(),
                });
            }
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![Token::Address(self.target), Token::Uint(self.value)])
    }
}

#[derive(Debug, Clone, Default)]
pub struct MultitransferArgs {
    pub transfers: Vec<Transfer>,
}

impl MultitransferArgs {
    pub fn new(transfers: Vec<Transfer>) -> Self {
        MultitransferArgs { transfers }
    }

    pub fn retain_sufficient(&mut self, fee_cost: U256) {
        loop {
            let transfers_len = self.transfers.len();
            let fee_cost_per_transfer = fee_cost / transfers_len;

            self.transfers.retain(|t| {
                if t.value < fee_cost_per_transfer {
                    log!(
                        "Transfer to {}, value: {} dropped due to insufficient balance",
                        t.target,
                        t.value
                    );
                    false
                } else {
                    true
                }
            });

            if self.transfers.len() == transfers_len {
                break;
            }
        }
    }
}

impl Tokenizable for MultitransferArgs {
    fn from_token(token: Token) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Token::Array(tokens) = token {
            let transfers = tokens
                .iter()
                .map(|t| Transfer::from_token(t.clone()))
                .collect::<Result<Vec<Transfer>, Error>>()?;

            return Ok(Self { transfers });
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        Token::Array(self.transfers.into_iter().map(|t| t.into_token()).collect())
    }
}

#[derive(Debug, Clone, Default)]
struct MulticallArgs {
    calls: Vec<Call>,
}

impl Tokenizable for MulticallArgs {
    fn from_token(token: Token) -> Result<Self, Error>
    where
        Self: Sized,
    {
        if let Token::Array(tokens) = token {
            let calls = tokens
                .iter()
                .map(|t| Call::from_token(t.clone()))
                .collect::<Result<Vec<Call>, Error>>()?;

            return Ok(Self { calls });
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        Token::Array(self.calls.into_iter().map(|c| c.into_token()).collect())
    }
}

impl MulticallArgs {
    pub fn new(calls: Vec<Call>) -> Self {
        MulticallArgs { calls }
    }
}

pub async fn multicall<T: Transport>(
    w3: &Web3Instance<T>,
    multicall_address: &str,
    from: String,
    mut calls: Vec<Call>,
    key_name: String,
    chain_id: u64,
    block_gas_limit: U256,
    gas_price: &U256,
) -> Result<Vec<MulticallResult>, MulticallError> {
    log!("[MULTICALL] chain: {}, multicall started", chain_id);

    let mut result: Vec<MulticallResult> = vec![];

    let contract_addr = address::to_h160(multicall_address)?;
    let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL_ABI)
        .map_err(|err| Web3Error::UnableToCreateContract(err.to_string()))?;

    let multicall_abi = ethabi::Contract::load(MULTICALL_ABI).unwrap();

    let event = multicall_abi
        .event(MULTICALL_EXECUTED_EVENT_NAME)
        .expect("should be able to get event by name");

    while !calls.is_empty() {
        let (current_calls_batch, _calls) = get_current_calls_batch(&calls, block_gas_limit);
        calls = _calls;

        result.append(
            &mut execute_multicall_batch(
                w3,
                from.clone(),
                &gas_price,
                &contract,
                MulticallArgs::new(current_calls_batch),
                chain_id,
                key_name.clone(),
                event,
            )
            .await?,
        );
    }

    Ok(result)
}

async fn execute_multicall_batch<T: Transport>(
    w3: &Web3Instance<T>,
    from: String,
    gas_price: &U256,
    contract: &Contract<T>,
    multicall_args: MulticallArgs,
    chain_id: u64,
    key_name: String,
    event: &Event,
) -> Result<Vec<MulticallResult>, MulticallError> {
    log!(
        "[MULTICALL] chain: {}, multicall batch started, calls: {}",
        chain_id,
        multicall_args.calls.len()
    );

    let options = Options {
        gas_price: Some(*gas_price),
        nonce: Some(w3.get_nonce(&from).await?),
        gas: Some(
            multicall_args
                .calls
                .iter()
                .fold(U256::from(BASE_GAS + GAS_FOR_OPS), |result, call| {
                    result + call.gas_limit
                }),
        ),
        ..Default::default()
    };

    log!("[MULTICALL] estimating gas for multicall");

    // TODO: maybe implement better gas estimation

    // let mut estimated_gas = Web3Instance::estimate_gas(
    //     contract,
    //     MULTICALL_CALL_FUNCTION,
    //     multicall_args.clone(),
    //     &from,
    //     &options,
    // )
    // .await?;

    // log!("[MULTICALL] gas_price: {}", gas_price);
    // log!("[MULTICALL] estimated gas: {}", estimated_gas);

    // let additional_gas = multicall_args
    //     .calls
    //     .iter()
    //     .fold(U256::from(0), |sum, c| sum + c.gas_limit);
    // TODO: ic users gas_limit will be too big, error
    // `RPC error: Error { code: ServerError(-32000), message: "tx fee (12.85 ether) exceeds the configured cap (1.00 ether)", data: None }`
    // will be returned

    // log!("[MULTICALL] additional gas: {}", additional_gas);

    // estimated_gas = estimated_gas + additional_gas;

    // options.gas = Some(estimated_gas);
    // log!(
    //     "[MULTICALL] chain: {}, estimated gas & limit: {}",
    //     chain_id,
    //     options.gas.unwrap()
    // );

    let params = vec![multicall_args.clone().into_token()];

    let signed_call = w3
        .sign(
            contract,
            MULTICALL_CALL_FUNCTION,
            params.clone(),
            options,
            from,
            key_name,
            chain_id,
        )
        .await?;

    for call in multicall_args.calls {
        log!(
            "[MULTICALL] chain: {}, call to: {}, user's gas: {}",
            chain_id,
            call.target,
            call.gas_limit
        );
    }

    log!("[MULTICALL] chain: {}, tx was signed", chain_id);

    let tx_hash = w3.send_raw_transaction_and_wait(signed_call).await?;

    log!("[MULTICALL] chain: {}, tx was executed", chain_id);

    let logs = w3
        .get_logs(
            tx_hash.block_number.expect("should be present").as_u64(),
            Some(tx_hash.block_number.expect("should be present").as_u64()),
            Some(vec![
                H256::from_str(MULTICALL_EXECUTED_TOPIC).expect("should be able to parse")
            ]),
            Some(contract.address()),
        )
        .await?;

    let mut multicall_results = Vec::new();

    for log in logs {
        let raw_log = RawLog {
            topics: log.topics,
            data: log.data.0,
        };

        let parsed_log = event
            .parse_log(raw_log)
            .map_err(|err| MulticallError::AbiParsingError(err.to_string()))?;

        for param in parsed_log.params {
            for multicall_result_token in param
                .value
                .into_array()
                .ok_or(MulticallError::InvalidMulticallResult)?
            {
                multicall_results.push(
                    MulticallResult::from_token(multicall_result_token)
                        .map_err(|err| MulticallError::FailedToParseFromLog(err.to_string()))?,
                );
            }
        }
    }

    Ok(multicall_results)
}

fn get_current_calls_batch(calls: &[Call], block_gas_limit: U256) -> (Vec<Call>, Vec<Call>) {
    let mut gas_counter = U256::from(BASE_GAS + 1000);
    for (i, call) in calls.iter().enumerate() {
        gas_counter += call.gas_limit;
        if gas_counter >= block_gas_limit {
            return (calls[..i].to_vec(), calls[i..].to_vec());
        }
    }

    (calls.to_vec(), vec![])
}

pub async fn estimate_multitransfer<T: Transport>(
    w3: &Web3Instance<T>,
    gas_price: U256,
    multitransfer_args: MultitransferArgs,
    multicall_address: &str,
    from: String,
) -> Result<U256, MulticallError> {
    let contract_addr = address::to_h160(multicall_address)?;
    let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL_ABI)
        .map_err(|err| Web3Error::UnableToCreateContract(err.to_string()))?;

    let nonce = w3.get_nonce(&from).await?;

    let options = Options {
        gas_price: Some(gas_price),
        nonce: Some(nonce),
        ..Default::default()
    };

    Ok(Web3Instance::estimate_gas(
        &contract,
        &MULTICALL_TRANSFER_FUNCTION,
        multitransfer_args.clone(),
        &from,
        &options,
    )
    .await?)
}

// TODO: reread this function and make sure it's correct
pub async fn multitransfer<T: Transport>(
    w3: &Web3Instance<T>,
    gas_price: U256,
    estimated_gas: U256,
    chain_id: u64,
    multitransfer_args: MultitransferArgs,
    multicall_address: &str,
    from: String,
    key_name: String,
) -> Result<(), MulticallError> {
    let contract_addr = address::to_h160(multicall_address)?;
    let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL_ABI)
        .map_err(|err| Web3Error::UnableToCreateContract(err.to_string()))?;

    let params = vec![multitransfer_args.clone().into_token()];

    let nonce = w3.get_nonce(&from).await?;

    let options = Options {
        gas_price: Some(gas_price),
        gas: Some(estimated_gas),
        nonce: Some(nonce),
        value: Some(
            multitransfer_args
                .transfers
                .iter()
                .fold(U256::from(0), |sum, t| sum + t.value),
        ),
        ..Default::default()
    };

    let signed_call = w3
        .sign(
            &contract,
            &MULTICALL_TRANSFER_FUNCTION,
            params,
            options,
            from,
            key_name,
            chain_id,
        )
        .await?;

    log!("[Multitransfer] tx send, chain_id: {}", chain_id);

    let tx = w3.send_raw_transaction_and_wait(signed_call).await?;
    log!("TX: {:?}", tx.transaction_hash);

    log!("[Multitransfer] tx received, chain_id: {}", chain_id);

    Ok(())
}
