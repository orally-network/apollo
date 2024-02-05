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

#[derive(Debug, Clone, Default)]
pub struct MultitransferArgs {
    transfers: Vec<Transfer>,
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

    while !calls.is_empty() {
        let (current_calls_batch, _calls) = get_current_calls_batch(&calls, block_gas_limit);
        calls = _calls;

        let results = execute_multicall_batch(
            w3,
            from.clone(),
            &gas_price,
            &contract,
            MulticallArgs::new(current_calls_batch),
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
    multicall_args: MulticallArgs,
    chain_id: u64,
    key_name: String,
) -> Result<Vec<Token>, MulticallError> {
    log!(
        "[MULTICALL] chain: {}, multicall batch started, calls: {}",
        chain_id,
        multicall_args.calls.len()
    );

    let mut options = Options {
        gas_price: Some(*gas_price),
        nonce: Some(w3.get_nonce(&from).await?),
        ..Default::default()
    };

    let estimated_gas = Web3Instance::estimate_gas(
        contract,
        MULTICALL_CALL_FUNCTION,
        multicall_args.clone(),
        &from,
        &options,
    )
    .await?;

    options.gas = Some(estimated_gas);

    // TODO: implement separate function for this

    // let estimatet_multicall_results = w3
    //     .get_call_result(
    //         contract,
    //         MULTICALL_CALL_FUNCTION,
    //         &params,
    //         address::to_h160(&from)?,
    //         Some(contract.address()),
    //         None,
    //     )
    //     .await?;

    // let mut filtered_batch: Vec<Call>;
    // for (result, call) in estimatet_multicall_results.iter().zip(batch) {
    //     let result = MulticallResult::from_token(result.clone())
    //         .map_err(|err| MulticallError::ContractError(err.to_string()))?;
    //     if result.used_gas <= call.gas_limit && result.used_gas <=  {}
    // }

    //

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
            "[MULTICALL] chain: {}, call to: {}, gas: {}",
            chain_id,
            call.target,
            call.gas_limit
        );
    }

    log!("[MULTICALL] chain: {}, tx was signed", chain_id);

    let tx_hash = w3.send_raw_transaction_and_wait(signed_call).await?;

    log!("[MULTICALL] chain: {}, tx was executed", chain_id);

    let call_result = w3
        .get_call_result(
            contract,
            MULTICALL_CALL_FUNCTION,
            &params,
            tx_hash.from,
            tx_hash.to,
            tx_hash.block_number,
        )
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
    multitransfer_args: MultitransferArgs,
    multicall_address: &str,
    from: String,
    key_name: String,
) -> Result<(), MulticallError> {
    let contract_addr = address::to_h160(multicall_address)?;
    let contract = Contract::from_json(w3.eth(), contract_addr, MULTICALL_ABI)
        .map_err(|err| Web3Error::UnableToCreateContract(err.to_string()))?;

    let params = vec![multitransfer_args.clone().into_token()];

    let gas_price = w3.get_gas_price().await?;
    let value = multitransfer_args
        .transfers
        .iter()
        .fold(U256::from(0), |sum, t| sum + t.value);

    let transfers_len = multitransfer_args.transfers.len();

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
        multitransfer_args,
        &from,
        &options,
    )
    .await?;

    options.value = Some(value - (gas_limit / transfers_len) * gas_price);
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
