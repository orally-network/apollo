use candid::{CandidType, Nat};
use thiserror::Error;

#[derive(Error, Debug, CandidType)]
pub enum ApolloError {
    #[error("ApolloInstanceError: {0}")]
    ApolloInstanceError(#[from] ApolloInstanceError),
    #[error("Utils error: {0}")]
    UtilsError(#[from] UtilsError),
}

#[derive(Error, Debug, CandidType)]
pub enum ApolloInstanceError {
    #[error("Failed to create: {0}")]
    FailedToCreate(String),
    #[error("Failed to install code: {0}")]
    FailedToInstallCode(String),
    #[error("Chain already exists: {0}")]
    ChainAlreadyExists(Nat),
    #[error("Timer is not initialized")]
    TimerIsNotInitialized,
    #[error("Utils error: {0}")]
    UtilsError(#[from] UtilsError),
    #[error("Balances error: {0}")]
    BalancesError(#[from] BalancesError),
    #[error("Web3 error: {0}")]
    Web3Error(#[from] Web3Error),
}

#[derive(Error, Debug, CandidType, PartialEq)]
pub enum UtilsError {
    #[error("Invalid address format: {0}")]
    InvalidAddressFormat(String),
}

#[derive(Error, Debug, CandidType)]
pub enum Web3Error {
    #[error("Unable to get gas_price: {0}")]
    UnableToGetGasPrice(String),
    #[error("Couldn't convert address to H160: {0}")]
    InvalidAddressFormat(String),
    #[error("Unable to get nonce: {0}")]
    UnableToGetNonce(String),
    #[error("Unable to estimate gas: {0}")]
    UnableToEstimateGas(String),
    #[error("Unable to sign contract call: {0}")]
    UnableToSignContractCall(String),
    #[error("Unable to execute raw transaction: {0}")]
    UnableToExecuteRawTx(String),
    #[error("Tx has failed")]
    TxHasFailed,
    #[error("Unable to get tx receipt: {0}")]
    UnableToGetTxReceipt(String),
    #[error("Tx timeout")]
    TxTimeout,
    #[error("Unable to form call data: {0}")]
    UnableToFormCallData(String),
    #[error("Unable to decode output: {0}")]
    UnableToDecodeOutput(String),
    #[error("Unable to call contract: {0}")]
    UnableToCallContract(String),
    #[error("Unable to create contract: {0}")]
    UnableToCreateContract(String),
}

#[derive(Error, Debug, CandidType)]
pub enum MulticallError {
    #[error("Invalid multicall result")]
    InvalidMulticallResult,
    #[error("Empty response")]
    EmptyResponse,
    #[error("Response is not an array, response: {0}")]
    ResponseIsNotAnArray(String),
    #[error("Utils error: {0}")]
    UtilsError(#[from] UtilsError),
    #[error("Web3 error: {0}")]
    Web3Error(#[from] Web3Error),
}

#[derive(Error, Debug, CandidType, PartialEq)]
pub enum BalancesError {
    #[error("Balance already exists")]
    BalanceAlreadyExists,
    #[error("Balance does not exist")]
    BalanceDoesNotExist,
    #[error("Nonce is too low")]
    NonceIsTooLow,
    #[error("Utils error: {0}")]
    UtilsError(#[from] UtilsError),
    #[error("Not enough funds")]
    NotEnoughFunds,
}
