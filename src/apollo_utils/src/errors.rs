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
}

#[derive(Error, Debug, CandidType)]
pub enum UtilsError {
    #[error("Invalid address format")]
    InvalidAddressFormat,
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
}
