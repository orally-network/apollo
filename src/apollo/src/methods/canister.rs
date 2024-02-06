use std::collections::HashMap;

use crate::methods::INIT_CYCLES_BALANCE;
use crate::types::UpdateMetadata;
use crate::types::{apollo_instance::ApolloInstance, Metadata, STATE};
use apollo_utils::apollo_instance::ApolloInstanceInit;
use apollo_utils::errors::{ApolloError, ApolloInstanceError};
use apollo_utils::get_metadata;
use apollo_utils::log;
use apollo_utils::memory::Cbor;
use apollo_utils::nat::ToNativeTypes;
use candid::{candid_method, encode_args, Nat};
use ic_cdk::api::management_canister::main::{
    create_canister, delete_canister, install_code, stop_canister, CanisterInstallMode,
    CreateCanisterArgument, InstallCodeArgument,
};
use ic_cdk::api::management_canister::provisional::CanisterIdRecord;
use ic_cdk::{query, update};

use crate::{AddApolloInstanceRequest, Result};

#[candid_method]
#[query]
fn get_metadata() -> Metadata {
    STATE.with(|s| s.borrow().metadata.get().0.clone())
}

#[candid_method]
#[update]
fn update_metadata(update_metadata_args: UpdateMetadata) -> Result<()> {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let mut metadata = state.metadata.get().0.clone();
        metadata.update(update_metadata_args);
        state.metadata.set(Cbor(metadata)).unwrap();
    });

    Ok(())
}

#[candid_method]
#[query]
fn get_apollo_instances() -> HashMap<u32, ApolloInstance> {
    STATE.with(|s| {
        s.borrow()
            .chains
            .iter()
            .map(|(k, v)| (k, v.0.clone()))
            .collect()
    })
}

#[candid_method]
#[update]
async fn add_apollo_instance(req: AddApolloInstanceRequest) -> Result<()> {
    log!(
        "Accepted {} cycles",
        ic_cdk::api::call::msg_cycles_accept128(INIT_CYCLES_BALANCE)
    );

    let chain_id = req.chain_id.clone();
    log!("Adding Chain: {}", chain_id);
    if STATE.with(|state| state.borrow().chains.contains_key(&chain_id.to_u32())) {
        return Err(ApolloError::ChainAlreadyExists(chain_id).into());
    }

    let canister_id = match create_canister(
        CreateCanisterArgument { settings: None },
        INIT_CYCLES_BALANCE,
    )
    .await
    {
        Ok((canister_id,)) => canister_id.canister_id,
        Err((_, error)) => {
            return Err(ApolloInstanceError::FailedToCreate(error.to_string()).into());
        }
    };

    let wasm = include_bytes!("../../../../assets/apollo_instance.wasm").to_vec();

    let payload = (ApolloInstanceInit {
        chain_id: chain_id.clone(),
        apollos_fee: req.apollos_fee,
        key_name: get_metadata!(key_name),
        chain_rpc: req.chain_rpc,
        apollo_coordinator: req.apollo_coordinator,
        multicall_address: req.multicall_address,
        timer_frequency_sec: req.timer_frequency_sec,
        block_gas_limit: req.block_gas_limit,
        sybil_canister_address: get_metadata!(sybil_canister_address),
        min_balance: req.min_balance,
    },);

    match install_code(InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id,
        wasm_module: wasm.to_vec(),
        arg: encode_args(payload).unwrap(),
    })
    .await
    {
        Ok(()) => {
            log!(
                "Code installed in apollo instance with chain_id: {}",
                chain_id
            );

            STATE.with(|s| {
                let mut state = s.borrow_mut();
                let apollo_instance = ApolloInstance {
                    canister_id,
                    is_active: true,
                    chain_id: chain_id.clone(),
                };

                state
                    .chains
                    .insert(chain_id.to_u32(), Cbor(apollo_instance));
            });
        }
        Err((_, error)) => {
            return Err(ApolloInstanceError::FailedToInstallCode(error.to_string()).into());
        }
    }

    log!("Chain Added: {}", chain_id);
    Ok(())
}

#[candid_method]
#[update]
// TODO: where did cycles go ?
async fn remove_apollo_instance(chain_id: Nat) -> Result<()> {
    log!("Removing Chain: {}", chain_id);

    let apollo_instance = crate::get_apollo_instance!(chain_id.clone());

    match stop_canister(CanisterIdRecord {
        canister_id: apollo_instance.canister_id,
    })
    .await
    {
        Ok(()) => {
            log!("Apollo instance stopped: {}", chain_id);
        }
        Err((_, error)) => {
            return Err(ApolloInstanceError::FailedToStop(error.to_string()).into());
        }
    }

    match delete_canister(CanisterIdRecord {
        canister_id: apollo_instance.canister_id,
    })
    .await
    {
        Ok(()) => {
            log!("Apollo instance removed: {}", chain_id);
        }
        Err((_, error)) => {
            return Err(ApolloInstanceError::FailedToDelete(error.to_string()).into());
        }
    };

    Ok(())
}

#[candid_method]
#[update]
pub async fn upgrade_chains() -> Result<()> {
    log!("Updating apollo instances");

    let wasm = include_bytes!("../../../../assets/apollo_instance.wasm").to_vec();
    let apollo_instances = get_apollo_instances();

    for (chain_id, apollo_instance) in apollo_instances {
        match install_code(InstallCodeArgument {
            mode: CanisterInstallMode::Upgrade,
            canister_id: apollo_instance.canister_id,
            wasm_module: wasm.to_vec(),
            arg: vec![],
        })
        .await
        {
            Ok(()) => {
                log!("Apollo instance upgraded: {}", chain_id);
            }
            Err((_, error)) => {
                return Err(ApolloInstanceError::FailedToUpgrade(error.to_string()).into());
            }
        }
    }

    Ok(())
}
