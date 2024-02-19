use apollo_utils::{
    address,
    errors::{ApolloInstanceError, Web3Error},
    get_metadata, log,
    nat::ToNatType,
    web3,
};
use candid::candid_method;
use ic_cdk::{query, update};

use crate::{
    jobs::withdraw,
    types::{allowances::Allowances, balances::Balances, timer::Timer, withdraw::WithdrawRequests},
    utils::apollo_evm_address,
    NatResult, Result,
};

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
pub fn get_balance(address: String) -> NatResult {
    Ok(Balances::get(&address).unwrap_or_default().amount)
}

/// Deposit amount to the AMA
///
/// # Arguments
///
/// * `tx_hash` - Hash of the transaction, where funds were transfered to the AMA
/// * `allowance` - Address of the contract, to whom grand permission to use funds
/// * `msg` - SIWE message, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
/// * `sig` - SIWE signature, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
///
/// # Returns
///
/// Returns a result that can contain an error message
#[candid_method]
#[update]
pub async fn deposit(
    tx_hash: String,
    allowance: Option<String>,
    msg: String,
    sig: String,
) -> Result<()> {
    let sender = apollo_utils::siwe::recover(msg, sig).await;

    let w3 = web3::instance(&get_metadata!(chain_rpc))?;

    let tx = w3.get_tx(&tx_hash).await?;

    let receiver = tx.to.ok_or(Web3Error::TxWithoutReceiver)?;

    let ama = address::to_h160(&apollo_evm_address().await?)?;

    if receiver != ama {
        return Err(ApolloInstanceError::TxWasNotSentToAMA);
    }

    Balances::save_nonce(&sender, &tx.nonce.to_nat())?;

    let amount = tx.value.to_nat();

    Balances::add_amount(&sender, &amount)?;

    if let Some(contract) = allowance {
        Allowances::grant(contract.clone(), sender.clone())?;
        log!("[ALLOWANCE] {sender} allowed {contract} to use his balance")
    }

    log!("[BALANCES] {sender} deposited amount {amount}");
    Ok(())
}

#[candid_method]
#[update]
pub async fn withdraw(receiver: String, msg: String, sig: String) -> Result<()> {
    let address = apollo_utils::siwe::recover(msg, sig).await;

    let amount = Balances::get(&receiver).unwrap_or_default().amount;

    WithdrawRequests::add(address.clone(), receiver, &amount)?;

    if !Timer::is_active() {
        withdraw::execute();
    }

    log!("[BALANCES] {address} withdrawed amount {amount}");

    Ok(())
}
