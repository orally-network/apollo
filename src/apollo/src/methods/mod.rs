use std::collections::HashMap;

use candid::{candid_method, Nat, encode_args};
use ic_cdk::{query, update, api::management_canister::{main::{create_canister, CreateCanisterArgument, install_code, InstallCodeArgument, CanisterInstallMode}, http_request::{TransformArgs, HttpResponse}}};
use ic_web3_rs::{Web3, transports::ICHttp, types::{BlockId, BlockNumber}};

use crate::{types::{Metadata, STATE, apollo_instance::ApolloInstance, candid_types::AddChainRequest}, log, utils::{nat::ToNativeTypes, set_custom_panic_hook}, memory::Cbor};
use crate::get_metadata;
use apollo_utils::errors::{ApolloError, ApolloInstanceError};



const INIT_CYCLES_BALANCE: u128 = 3_000_000_000_000; 


#[candid_method]
#[query]
fn get_metadata() -> Metadata {
    log!("Metadata");
    STATE.with(|s| {
        s.borrow().metadata.get().0.clone()
    })
}

#[candid_method]
#[query]
fn get_chains() -> HashMap<u32, ApolloInstance> {
    log!("Chains");
    STATE.with(|s| {
        log!("ABOBA");
        s.borrow().chains.iter().map(|(k, v)| (k.clone(), v.0.clone())).collect()
    })
} 



#[candid_method]
#[update]
async fn add_chain(req: AddChainRequest) -> Result<(), ApolloError> {
    log!("Adding Chain: {}", req.chain_id);
    if STATE.with(|state| state.borrow().chains.contains_key(&req.chain_id.to_u32())) {
        return Err(ApolloInstanceError::ChainAlreadyExists(req.chain_id).into());
    }


    let canister_id = match create_canister(CreateCanisterArgument { settings: None }, INIT_CYCLES_BALANCE.into()).await {
        Ok((canister_id,)) => canister_id.canister_id,
        Err((_, error)) => {
            return Err(ApolloInstanceError::FailedToCreate(error.to_string()).into());
        }
    };


    let wasm = include_bytes!("../../../../apollo_instance.wasm").to_vec();

    let payload = (get_metadata!(tx_fee), get_metadata!(key_name), req.chain_id.clone(), req.chain_rpc.clone(), req.timer_frequency);


    match install_code(InstallCodeArgument {
        mode: CanisterInstallMode::Install,
        canister_id: canister_id.clone(),
        wasm_module: wasm.to_vec(),
        arg: encode_args(payload).unwrap(),
    }).await {
        Ok(()) => {
            log!("Code installed in apollo instance with chain_id: {}", req.chain_id);
            STATE.with(|s| {
                let mut state = s.borrow_mut();
                let apollo_instance = ApolloInstance {
                    canister_id: canister_id.clone(),
                    is_active: true,
                };

                state.chains.insert(req.chain_id.to_u32(), Cbor(apollo_instance));
            });
        },
        Err((_, error)) => {
            return Err(ApolloInstanceError::FailedToInstallCode(error.to_string()).into());
        }
    }


    log!("Chain Added: {}", req.chain_id);
    Ok(())
}



#[query]
fn transform(response: TransformArgs) -> HttpResponse {
    HttpResponse {
        status: response.response.status,
        body: response.response.body,
        headers: Vec::new(),
    }
}


#[candid_method]
#[update]
async fn test () -> Result<(), ApolloError> {
    log!("Test");

    let w3 = Web3::new(ICHttp::new("https://goerli.infura.io/v3/8e4147cd4995430182a09781136f8745", None).unwrap());


    // w3.eth.transport().

    let block = w3.eth().block(BlockId::Number(BlockNumber::Latest)).await.unwrap();
    log!("Block: {:?}", block);

    


    Ok(())
}


#[ic_cdk::init]
fn init(tx_fee: Nat, key_name: String, timer_frequency: Nat) {
    set_custom_panic_hook();

    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.metadata.set(Cbor(Metadata {
            tx_fee,
            key_name,
            timer_frequency,
            ..Default::default()
        })).unwrap();
    });
}

