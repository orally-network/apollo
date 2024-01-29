use ic_web3_rs::{
    contract::{tokens::Tokenizable, Contract, Error, Options},
    ethabi::Token,
    types::{H160, U256},
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
pub const BASE_GAS: u64 = 27_000;
pub const GAS_PER_TRANSFER: u64 = 7_900;
const GAS_FOR_OPS: u64 = 10_000;

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
                return Ok(Self { target, value });
            }
        }

        Err(Error::InvalidOutputType("invalid tokens".into()))
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![Token::Address(self.target), Token::Uint(self.value)])
    }
}

pub async fn multicall<T: Transport>(
    w3: &Web3Instance<T>,
    multicall_address: &str,
    from: String,
    calls: Vec<Call>,
    key_name: String,
    chain_id: u64,
    gas_price: U256,
    block_gas_limit: U256,
) -> Result<Vec<MulticallResult>, MulticallError> {
    let mut calls = calls;
    let mut result: Vec<MulticallResult> = vec![];

    let contract_addr = address::to_h160(multicall_address)?;
    let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL_ABI)
        .map_err(|err| Web3Error::UnableToCreateContract(err.to_string()))?;

    while !calls.is_empty() {
        let (current_calls_batch, _calls) = get_current_calls_batch(&calls, block_gas_limit);
        calls = _calls;

        let results = execute_multicall_batch(
            w3,
            from.clone(),
            &gas_price,
            &contract,
            &current_calls_batch,
            chain_id,
            key_name.clone(),
        )
        .await?;

        result.append(
            &mut results
                .iter()
                .map(|token| {
                    MulticallResult::from_token(token.clone()).expect("failed to decode from token")
                })
                .collect::<Vec<MulticallResult>>(),
        );
    }

    Ok(result)
}

async fn execute_multicall_batch<T: Transport>(
    w3: &Web3Instance<T>,
    from: String,
    gas_price: &U256,
    contract: &Contract<T>,
    batch: &[Call],
    chain_id: u64,
    key_name: String,
) -> Result<Vec<Token>, MulticallError> {
    let options = Options {
        gas_price: Some(*gas_price),
        gas: Some(
            batch
                .iter()
                .fold(U256::from(BASE_GAS + GAS_FOR_OPS), |result, call| {
                    result + call.gas_limit
                }),
        ),
        nonce: Some(w3.get_nonce(&from).await?),
        ..Default::default()
    };

    let params: Vec<Token> = batch.iter().map(|c| c.clone().into_token()).collect();

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

    log!("[MULTICALL] chain: {}, tx was signed", chain_id);

    let tx_hash = w3.send_raw_transaction_and_wait(signed_call).await?;

    log!("[MULTICALL] chain: {}, tx was executed", chain_id);

    let call_result = w3
        .get_call_result(contract, MULTICALL_CALL_FUNCTION, &params, tx_hash)
        .await?;

    let token = call_result.first().ok_or(MulticallError::EmptyResponse)?;

    token
        .clone()
        .into_array()
        .ok_or(MulticallError::ResponseIsNotAnArray(token.to_string()).into())
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

// TODO: reread this function and make sure it's correct
pub async fn multitransfer<T: Transport>(
    w3: &Web3Instance<T>,
    chain_id: u64,
    transfers: Vec<Transfer>,
    multicall_address: &str,
    from: String,
    key_name: String,
) -> Result<(), MulticallError> {
    let contract_addr = address::to_h160(multicall_address)?;
    let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL_ABI)
        .map_err(|err| Web3Error::UnableToCreateContract(err.to_string()))?;

    let params: Vec<Token> = transfers.iter().map(|c| c.clone().into_token()).collect();

    let gas_price = w3.get_gas_price().await?;
    let value = transfers.iter().fold(U256::from(0), |sum, t| sum + t.value);
    let nonce = w3.get_nonce(&from).await?;

    let mut options = Options {
        gas_price: Some(gas_price),
        value: Some(value),
        nonce: Some(nonce),
        ..Default::default()
    };

    let gas_limit = Web3Instance::estimate_gas(
        &contract,
        &MULTICALL_TRANSFER_FUNCTION,
        &params,
        &from,
        &options,
    )
    .await?;

    options.value = Some(value - (gas_limit / transfers.len()) * gas_price);
    options.gas = Some(gas_limit);

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

    w3.send_raw_transaction_and_wait(signed_call).await?;

    log!("[Multitransfer] tx received, chain_id: {}", chain_id);

    Ok(())
}
