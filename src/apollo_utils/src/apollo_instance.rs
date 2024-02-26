use candid::{CandidType, Nat};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, CandidType, Serialize, Deserialize)]
pub struct ApolloInstanceInit {
    pub apollos_fee: Nat,
    pub key_name: String,
    pub chain_id: Nat,
    pub chain_rpc: String,
    pub apollo_coordinator: String,
    pub multicall_address: String,
    pub timer_frequency_sec: u64,
    pub block_gas_limit: Nat,
    pub sybil_canister_address: String,
    pub evm_rpc_canister: String,
    pub min_balance: Nat,
}

#[derive(Serialize, Debug, Deserialize, CandidType, Clone)]
pub struct ApolloInstanceMetadata {
    pub apollos_fee: Nat,
    pub key_name: String,
    pub chain_id: Nat,
    pub chain_rpc: String,
    pub apollo_coordinator: String,
    pub apollo_evm_address: Option<String>,
    pub multicall_address: String,
    pub sybil_canister_address: String, // Principal is not supported by ciborium
    pub evm_rpc_canister: String,       // Principal is not supported by ciborium
    pub block_gas_limit: Nat,
    pub min_balance: Nat,
}

#[derive(Serialize, Deserialize, CandidType, Clone)]
pub struct UpdateMetadata {
    pub apollos_fee: Option<Nat>,
    pub chain_id: Option<Nat>,
    pub chain_rpc: Option<String>,
    pub apollo_coordinator: Option<String>,
    pub multicall_address: Option<String>,
    pub sybil_canister_address: Option<String>, // Principal is not supported by ciborium
    pub evm_rpc_canister: Option<String>,       // Principal is not supported by ciborium
    pub block_gas_limit: Option<Nat>,
    pub min_balance: Option<Nat>,
}

impl ApolloInstanceMetadata {
    pub fn update(&mut self, update: UpdateMetadata) {
        if let Some(apollos_fee) = update.apollos_fee {
            self.apollos_fee = apollos_fee;
        }
        if let Some(chain_id) = update.chain_id {
            self.chain_id = chain_id;
        }
        if let Some(chain_rpc) = update.chain_rpc {
            self.chain_rpc = chain_rpc;
        }
        if let Some(apollo_coordinator) = update.apollo_coordinator {
            self.apollo_coordinator = apollo_coordinator;
        }
        if let Some(multicall_address) = update.multicall_address {
            self.multicall_address = multicall_address;
        }
        if let Some(sybil_canister_address) = update.sybil_canister_address {
            self.sybil_canister_address = sybil_canister_address;
        }
        if let Some(evm_rpc_canister) = update.evm_rpc_canister {
            self.evm_rpc_canister = evm_rpc_canister;
        }
        if let Some(block_gas_limit) = update.block_gas_limit {
            self.block_gas_limit = block_gas_limit;
        }
        if let Some(min_balance) = update.min_balance {
            self.min_balance = min_balance;
        }
    }
}

impl Default for ApolloInstanceMetadata {
    fn default() -> Self {
        Self {
            apollos_fee: Nat::from(0),
            key_name: "".to_string(),
            chain_id: Nat::from(0),
            chain_rpc: "".to_string(),
            apollo_coordinator: "".to_string(),
            apollo_evm_address: None,
            multicall_address: "".to_string(),
            sybil_canister_address: "".to_string(),
            evm_rpc_canister: "".to_string(),
            block_gas_limit: Nat::from(0),
            min_balance: Nat::from(0),
        }
    }
}

// Used to generate metadata from ApolloInstanceInit
impl From<ApolloInstanceInit> for ApolloInstanceMetadata {
    fn from(init: ApolloInstanceInit) -> Self {
        Self {
            apollos_fee: init.apollos_fee,
            key_name: init.key_name,
            chain_id: init.chain_id,
            chain_rpc: init.chain_rpc,
            apollo_coordinator: init.apollo_coordinator,
            apollo_evm_address: None,
            multicall_address: init.multicall_address,
            block_gas_limit: init.block_gas_limit,
            sybil_canister_address: init.sybil_canister_address,
            evm_rpc_canister: init.evm_rpc_canister,
            min_balance: init.min_balance,
        }
    }
}
