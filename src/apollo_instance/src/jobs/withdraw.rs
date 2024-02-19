use std::str::FromStr;

use anyhow::Result;
use apollo_utils::{
    get_metadata, log,
    multicall::{self, MultitransferArgs, Transfer},
    nat::{ToNatType, ToNativeTypes},
    web3,
};
use ic_web3_rs::types::{H160, U256};

use crate::{
    types::{
        balances::Balances,
        withdraw::{WithdrawRequest, WithdrawRequests},
    },
    utils::apollo_evm_address,
};

const MAX_TRANSFERS: usize = 100;

pub fn execute() {
    ic_cdk::spawn(withdraw())
}

pub async fn withdraw() {
    let reqs = WithdrawRequests::get_all();
    if reqs.is_empty() {
        return;
    }

    log!("[WITHDRAWER] withdraw job started");

    if let Err(err) = send_funds(&reqs).await {
        log!("[WITHDRAWER] Error while sending funds: {err}");
    }

    log!("[WITHDRAWER] withdraw job executed");
}

// Transaction fee will be reduced from the amount sent.
// Meaning if user wants to send 1 ETH, and the transaction fee is 0.01 ETH, the user will send 0.99 ETH.
async fn send_funds(reqs: &[WithdrawRequest]) -> Result<()> {
    if reqs.is_empty() {
        return Ok(());
    }

    let transfers: Vec<Transfer> = reqs
        .iter()
        .map(|req| Transfer {
            target: H160::from_str(&req.receiver).expect("should be valid address"),
            value: req.amount.to_u256(),
            from: req.from.clone(),
        })
        .collect();

    log!("Transfers: {:#?}", transfers);

    let w3 = web3::instance(&get_metadata!(chain_rpc))?;

    for transfers_chunk in transfers.chunks(MAX_TRANSFERS) {
        // multiply the gas_price to 1.2 to avoid long transaction confirmation
        let gas_price: U256 = (w3.get_gas_price().await? / 10) * 12;

        let mut multitransfer_args = MultitransferArgs::new(transfers_chunk.to_vec());

        let gas = multicall::estimate_multitransfer(
            &w3,
            gas_price.clone(),
            multitransfer_args.clone(),
            &get_metadata!(multicall_address),
            apollo_evm_address().await?,
        )
        .await?;

        multitransfer_args.retain_sufficient(gas * gas_price);

        multicall::multitransfer(
            &w3,
            gas_price,
            gas,
            get_metadata!(chain_id).to_u64(),
            multitransfer_args.clone(),
            &get_metadata!(multicall_address),
            apollo_evm_address().await?,
            get_metadata!(key_name),
        )
        .await?;

        for transfer in multitransfer_args.transfers {
            Balances::reduce_amount(&transfer.from, &transfer.value.to_nat())?;
        }
    }

    WithdrawRequests::clean()?;

    Ok(())
}
