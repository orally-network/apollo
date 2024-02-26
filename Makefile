
all: local_deploy_apollo

update_candid:
	cargo test update_candid
	dfx generate 

build_apollo_instance:
	dfx build --check apollo_instance
	mv ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm assets/apollo_instance.wasm


local_deploy_apollo: update_candid build_apollo_instance   
ifndef SYBIL_CANISTER
	$(error SYBIL_CANISTER ENV is undefined)
endif
	dfx deploy --specified-id ajuq4-ruaaa-aaaaa-qaaga-cai evm_rpc --argument '(record { nodesInSubnet = 28 })'

	dfx canister create apollo && dfx build apollo && gzip -f -1 ./.dfx/local/canisters/apollo/apollo.wasm
	dfx canister install --wasm ./.dfx/local/canisters/apollo/apollo.wasm.gz --argument \
		"(\"${SYBIL_CANISTER}\", \"dfx_test_key\")" apollo

local_deploy_apollo_instance: update_candid 
ifndef SYBIL_CANISTER
	$(error SYBIL_CANISTER ENV is undefined)
endif

ifndef MULTICALL_ADDRESS
	$(eval MULTICALL_ADDRESS := 0x65309C2B0f31866a46b0FB2BcA2c3188a747B78f)
	$(echo MULTICALL_ADDRESS ENV is undefined, using default value: ${MULTICALL_ADDRESS} for holeski)
endif

	dfx deploy --specified-id ajuq4-ruaaa-aaaaa-qaaga-cai evm_rpc --argument '(record { nodesInSubnet = 28 })'

	dfx canister create apollo_instance && dfx build apollo_instance && gzip -f -1 ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm
	dfx canister install --wasm ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm.gz --argument \
		"(record {\
				apollos_fee = 0:nat; \
				key_name = \"dfx_test_key\"; \
				chain_id = 167008:nat; \
				chain_rpc = \"https://taiko-katla.blockpi.network/v1/rpc/public\"; \
				apollo_coordinator = \"0xC1e42d86716f8b8fA616249112a21622b07319a3\"; \
				multicall_address = \"${MULTICALL_ADDRESS}\"; \
				timer_frequency_sec = 10:nat64; \
				block_gas_limit = 1000000000000:nat; \
				sybil_canister_address = \"${SYBIL_CANISTER}\"; \
				min_balance = 50_000_000_000_000_000:nat; \
			})" apollo_instance

local_upgrade_apollo_instance: update_candid 
	dfx build apollo_instance 
	gzip -f -1 ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm
	dfx canister install --mode upgrade --wasm ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm.gz apollo_instance



local_upgrade: local_upgrade_apollo 

local_upgrade_apollo: build_apollo_instance update_candid 
	dfx build apollo 
	gzip -f -1 ./.dfx/local/canisters/apollo/apollo.wasm
	dfx canister install --mode upgrade --wasm ./.dfx/local/canisters/apollo/apollo.wasm.gz apollo
	dfx canister call apollo upgrade_chains 


ic_upgrade: ic_upgrade_apollo


ic_upgrade_apollo: build_apollo_instance update_candid
	dfx build apollo --network ic && gzip -f -1 ./.dfx/ic/canisters/apollo/apollo.wasm
	dfx canister install --mode upgrade --wasm ./.dfx/ic/canisters/apollo/apollo.wasm.gz --network ic apollo
	dfx canister call apollo upgrade_chains --ic 

ic_deploy_apollo: build_apollo_instance update_candid 
ifndef SYBIL_CANISTER
	$(error SYBIL_CANISTER ENV is undefined)
endif
	dfx canister create apollo
	dfx build apollo && gzip -f -1 ./.dfx/local/canisters/apollo/apollo.wasm
	dfx canister install --wasm ./.dfx/local/canisters/apollo/apollo.wasm.gz --argument \
		"(\"${SYBIL_CANISTER}\", \"key_1\")" apollo --ic



local_apollo_add_apollo_instance: local_upgrade_apollo
ifndef MULTICALL_ADDRESS
	$(eval MULTICALL_ADDRESS := 0x65309C2B0f31866a46b0FB2BcA2c3188a747B78f)
	$(echo MULTICALL_ADDRESS ENV is undefined, using default value: ${MULTICALL_ADDRESS} for holeski)
endif

ifndef CHAIN_RPC
	$(error CHAIN_RPC ENV is undefined)
endif

ifndef APOLLO_COORDINATOR
	$(error APOLLO_COORDINATOR ENV is undefined)
endif


	dfx canister call apollo add_apollo_instance \
		"(record {\
			apollos_fee = 0:nat; \
			chain_id = 11155111:nat; \
			chain_rpc = \"${CHAIN_RPC}\"; \
			apollo_coordinator = \"${APOLLO_COORDINATOR}\"; \
			multicall_address = \"${MULTICALL_ADDRESS}\"; \
			timer_frequency_sec = 10:nat64; \
			block_gas_limit = 1_000_000_000_000:nat; \
			evm_rpc_canister = \"$(shell dfx canister id evm_rpc)\"; \
			min_balance = 50_000_000_000_000_000:nat; \
		})" --with-cycles 500000000000 --wallet $(shell dfx identity get-wallet) 


