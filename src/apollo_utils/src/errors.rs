use candid::{CandidType, Nat};
use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug, CandidType, Deserialize)]
pub enum ApolloError {
    #[error("Chain not found: {0}")]
    ChainNotFound(Nat),
    #[error("Chain already exists: {0}")]
    ChainAlreadyExists(Nat),
    #[error("Communication with Apollo instance failed: {0}")]
    CommunicationWithApolloInstanceFailed(String),
    #[error("Failed to get canister status: {0}")]
    FailedToGetCanisterStatus(String),
    #[error("ApolloInstanceError: {0}")]
    ApolloInstanceError(#[from] ApolloInstanceError),
    #[error("Utils error: {0}")]
    UtilsError(#[from] UtilsError),
}

#[derive(Error, Debug, CandidType, Deserialize)]
pub enum ApolloInstanceError {
    #[error("Failed to update settings: {0}")]
    FailedToUpdateSettings(String),
    #[error("Failed to send cycles: {0}")]
    FailedToSendCycles(String),
    #[error("Failed to create: {0}")]
    FailedToCreate(String),
    #[error("Failed to stop: {0}")]
    FailedToStop(String),
    #[error("Failed to delete: {0}")]
    FailedToDelete(String),
    #[error("Failed to install code: {0}")]
    FailedToInstallCode(String),
    #[error("Failed to upgrade: {0}")]
    FailedToUpgrade(String),
    #[error("Utils error: {0}")]
    UtilsError(#[from] UtilsError),
    #[error("Balances error: {0}")]
    BalancesError(#[from] BalancesError),
    #[error("WithdrawRequests error: {0}")]
    WithdrawRequestsError(#[from] WithdrawRequestsError),
    #[error("Web3 error: {0}")]
    Web3Error(#[from] Web3Error),
    #[error("Tx was not sent to Apollo main address")]
    TxWasNotSentToAMA,
    #[error("Apollo coordinator pooling error: {0}")]
    ApolloCoordinatorPoolingError(String),
    #[error("Failed to restart timer: {0}")]
    FailedToRestartTimer(String),
}

#[derive(Error, Debug, CandidType, PartialEq, Deserialize)]
pub enum UtilsError {
    #[error("Invalid address format: {0}")]
    InvalidAddressFormat(String),
    #[error("From hex error: {0}")]
    FromHexError(String),
    #[error("Failed to get apollo evm address: {0}")]
    FailedToGetApolloEvmAddress(String),
    #[error("Not a controller")]
    NotAController,
}

#[derive(Error, Debug, CandidType, Deserialize)]
pub enum LogsPoolingError {
    #[error("Web3 error: {0}")]
    Web3Error(#[from] Web3Error),
    #[error("Abi parsing error: {0}")]
    AbiParsingError(String),
    #[error("Failed to process requests: {0}")]
    FailedToProcessRequests(String),
    #[error("Utils error: {0}")]
    UtilsError(#[from] UtilsError),
}

#[derive(Error, Debug, CandidType, Deserialize)]
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
    #[error("Unable to get tx receipt: {0}")]
    UnableToGetTxReceipt(String),
    #[error("Tx timeout")]
    TxTimeout,
    #[error("Tx not found")]
    TxNotFound,
    #[error("Tx without receiver")]
    TxWithoutReceiver,
    #[error("Tx has failed")]
    TxHasFailed,
    #[error("Unable to get block number: {0}")]
    UnableToGetBlockNumber(String),
    #[error("Unable to get logs: {0}")]
    UnableToGetLogs(String),
    #[error("Unable to form call data: {0}")]
    UnableToFormCallData(String),
    #[error("Unable to decode output: {0}")]
    UnableToDecodeOutput(String),
    #[error("Unable to call contract: {0}")]
    UnableToCallContract(String),
    #[error("Unable to create contract: {0}")]
    UnableToCreateContract(String),
    #[error("Utils error: {0}")]
    UtilsError(#[from] UtilsError),
}

#[derive(Error, Debug, CandidType, Deserialize)]
pub enum MulticallError {
    #[error("Invalid multicall result")]
    InvalidMulticallResult,
    #[error("Empty response")]
    EmptyResponse,
    #[error("Response is not an array, response: {0}")]
    ResponseIsNotAnArray(String),
    #[error("Abi parsing error: {0}")]
    AbiParsingError(String),
    #[error("Utils error: {0}")]
    UtilsError(#[from] UtilsError),
    #[error("Web3 error: {0}")]
    Web3Error(#[from] Web3Error),
    #[error("Failed to parse multicall result from log: {0}")]
    FailedToParseFromLog(String),
    #[error("Unable to encode call data: {0}")]
    UnableToEncodeCallData(String),
    #[error("Block gas limit is too low")]
    BlockGasLimitIsTooLow,
}

#[derive(Error, Debug, CandidType, PartialEq, Deserialize)]
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

#[derive(Error, Debug, CandidType, PartialEq, Deserialize)]
pub enum WithdrawRequestsError {
    #[error("Unable to add withdraw request: {0}")]
    UnableToAddWithdrawRequest(String),
    #[error("Unable to clean withdraw requests: {0}")]
    UnableToCleanWithdrawRequests(String),
    #[error("Utils error: {0}")]
    UtilsError(#[from] UtilsError),
}

#[derive(Error, Debug, CandidType, PartialEq, Deserialize)]
pub enum SybilError {
    #[error("Unsuppored Asset Data Type: {0}")]
    UnsupportedAssetDataType(String),
    #[error("Canister error: {0}")]
    CanisterError(String),
    #[error("Invalid principal: {0}")]
    InvalidPrincipal(String),
}
