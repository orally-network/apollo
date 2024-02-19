use crate::{types::allowances::Allowances, Result};
use apollo_utils::log;
use candid::candid_method;
use ic_cdk::update;

/// Allow smartcontract use funds from the user's balance
///
/// # Arguments
///
/// * `address` - Address of the contract
/// * `msg` - SIWE message, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
/// * `sig` - SIWE signature, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
///
/// # Returns
///
/// Returns a result that can contain an error message

#[candid_method]
#[update]
pub async fn grant(address: String, msg: String, sig: String) -> Result<()> {
    let user = apollo_utils::siwe::recover(msg, sig).await;

    Allowances::grant(address.clone(), user.clone())?;

    log!("[ALLOWANCE] {user} allowed {address} to use his balance");
    Ok(())
}

/// Restrict smartcontract from using funds from the user's balance
///
/// # Arguments
///
/// * `address` - Address of the contract
/// * `msg` - SIWE message, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
/// * `sig` - SIWE signature, For more information, refer to the [SIWE message specification](https://eips.ethereum.org/EIPS/eip-4361)
///
/// # Returns
///
/// Returns a result that can contain an error message

#[candid_method]
#[update]
pub async fn restrict(address: String, msg: String, sig: String) -> Result<()> {
    let user = apollo_utils::siwe::recover(msg, sig).await;

    Allowances::restrict(address.clone(), user.clone())?;

    log!("[ALLOWANCE] {user} restricted {address} from using his balance");
    Ok(())
}
