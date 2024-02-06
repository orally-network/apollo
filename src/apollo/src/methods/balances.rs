use apollo_utils::errors::{ApolloError, ApolloInstanceError};
use apollo_utils::retry_until_success;
use candid::{candid_method, Nat};
use ic_cdk::{query, update};

use crate::Result;

/// Deposit amount to the AMA
///
/// # Arguments
///
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `tx_hash` - Hash of the transaction, where funds were transfered to the AMA
/// * `msg` - SIWE message, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
/// * `sig` - SIWE signature, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
///
/// # Returns
///
/// Returns a result that can contain an error message
#[candid_method]
#[update]
pub async fn deposit(chain_id: Nat, tx_hash: String, msg: String, sig: String) -> Result<()> {
    let apollo_instance = crate::get_apollo_instance!(chain_id);

    let (result,): (std::result::Result<(), ApolloInstanceError>,) =
        retry_until_success!(ic_cdk::call(
            apollo_instance.canister_id,
            "deposit",
            (tx_hash.clone(), msg.clone(), sig.clone())
        ))
        .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

    Ok(result?)
}

/// Get balance of the user
///
/// # Arguments
/// * `chain_id` - Unique identifier of the chain, for example Ethereum Mainnet is 1
/// * `address` - Address of the user, for example 0x1234567890abcdef1234567890abcdef12345678
///
/// # Returns
///
/// Returns a result with address's balance
#[candid_method]
#[query]
pub async fn get_balance(chain_id: Nat, address: String) -> Result<Nat> {
    let apollo_instance = crate::get_apollo_instance!(chain_id);

    let (result,): (std::result::Result<Nat, ApolloInstanceError>,) =
        retry_until_success!(ic_cdk::call(
            apollo_instance.canister_id,
            "get_balance",
            (address.clone(),)
        ))
        .map_err(|(_, msg)| ApolloError::CommunicationWithApolloInstanceFailed(msg))?;

    Ok(result?)
}
