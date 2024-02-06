use apollo_utils::{
    address,
    errors::{ApolloInstanceError, Web3Error},
    get_metadata, log,
    nat::ToNatType,
    web3,
};
use candid::{candid_method, Nat};
use ic_cdk::{query, update};

use crate::{types::balances::Balances, utils::apollo_evm_address, Result};

/// Get balance of the user
///
/// # Arguments
/// * `address` - Address of the user, for example 0x1234567890abcdef1234567890abcdef12345678
///
/// # Returns
///
/// Returns a result with address's balance
#[candid_method]
#[query]
pub fn get_balance(address: String) -> Result<Nat> {
    Ok(Balances::get(&address).unwrap_or_default().amount)
}

/// Deposit amount to the AMA
///
/// # Arguments
///
/// * `tx_hash` - Hash of the transaction, where funds were transfered to the AMA
/// * `msg` - SIWE message, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
/// * `sig` - SIWE signature, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
///
/// # Returns
///
/// Returns a result that can contain an error message
#[candid_method]
#[update]
pub async fn deposit(tx_hash: String, msg: String, sig: String) -> Result<()> {
    let address = apollo_utils::siwe::recover(msg, sig).await;

    let w3 = web3::instance(&get_metadata!(chain_rpc))?;

    let tx = w3.get_tx(&tx_hash).await?;

    let receiver = tx.to.ok_or(Web3Error::TxWithoutReceiver)?;

    let ama = address::to_h160(&apollo_evm_address().await?)?;

    if receiver != ama {
        return Err(ApolloInstanceError::TxWasNotSentToAMA);
    }

    Balances::save_nonce(&address, &tx.nonce.to_nat())?;

    let amount = tx.value.to_nat();

    Balances::add_amount(&address, &amount)?;

    log!("[BALANCES] {address} deposited amount {amount}");
    Ok(())
}
