use candid::Principal;
use ic_web3_rs::{
    ethabi::Address,
    ic::{get_public_key, pubkey_to_address},
};

/// get canister's eth address
/// TODO: delete ? 
pub async fn get_eth_addr(
    canister_id: Option<Principal>,
    derivation_path: Option<Vec<Vec<u8>>>,
    name: String,
) -> Result<Address, String> {
    let path = if let Some(v) = derivation_path {
        v
    } else {
        vec![ic_cdk::id().as_slice().to_vec()]
    };

    match get_public_key(canister_id, path, name).await {
        Ok(pubkey) => pubkey_to_address(&pubkey),
        Err(e) => Err(e),
    }
}
