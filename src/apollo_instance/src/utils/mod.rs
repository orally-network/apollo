use crate::log;
use anyhow::{anyhow, Result};
use apollo_utils::{address, canister::get_eth_addr, get_metadata, update_metadata};

pub fn set_custom_panic_hook() {
    _ = std::panic::take_hook(); // clear custom panic hook and set default
    let old_handler = std::panic::take_hook(); // take default panic hook

    // set custom panic hook
    std::panic::set_hook(Box::new(move |info| {
        log!("PANIC OCCURRED: {:#?}", info);
        old_handler(info);
    }));
}

pub async fn apollo_evm_address() -> Result<String> {
    if let Some(address) = get_metadata!(apollo_evm_address) {
        return Ok(address);
    }

    let addr = get_eth_addr(None, Some(vec![vec![]]), get_metadata!(key_name))
        .await
        .map(|addr| address::from_h160(&addr))
        .map_err(|e| anyhow!("{e}"))?;

    update_metadata!(apollo_evm_address, Some(addr.clone()));

    Ok(addr)
}
