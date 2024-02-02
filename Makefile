all: local_deploy_apollo

update_candid:
	cargo test update_candid
	dfx generate 

build_apollo_instance:
	dfx build --check apollo_instance
	mv ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm assets/apollo_instance.wasm


local_deploy_apollo: update_candid build_apollo_instance   
	dfx canister create apollo && dfx build apollo && gzip -f -1 ./.dfx/local/canisters/apollo/apollo.wasm
	dfx canister install --wasm ./.dfx/local/canisters/apollo/apollo.wasm.gz --argument \
		"(0:nat, \"dfx_test_key\", 10:nat)" apollo

local_deploy_apollo_instance: update_candid 

ifndef SYBIL_CANISTER
	$(error SYBIL_CANISTER ENV is undefined)
endif

ifndef MULTICALL_ADDRESS
	$(eval MULTICALL_ADDRESS := "0x65309C2B0f31866a46b0FB2BcA2c3188a747B78f")
	$(echo MULTICALL_ADDRESS ENV is undefined, using default value: ${MULTICALL_ADDRESS} for holeski)
endif

	dfx canister create apollo_instance && dfx build apollo_instance && gzip -f -1 ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm
	dfx canister install --wasm ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm.gz --argument \
		"(record {\
			apollos_fee = 0:nat; \
			key_name = \"dfx_test_key\"; \
			chain_id = 17000:nat; \
			chain_rpc = \"https://l1rpc.katla.taiko.xyz\"; \
			apollo_coordinator = \"0xC1e42d86716f8b8fA616249112a21622b07319a3\"; \
			multicall_address = \"${MULTICALL_ADDRESS}\"; \
			timer_frequency = 30:nat64; \
			block_gas_limit = 1000000000000:nat64; \
			sybil_canister_address = principal \"${SYBIL_CANISTER}\"; \ 
			min_balance = 100_000_000_000_000_000; \
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


ic_upgrade: ic_upgrade_apollo


ic_upgrade_apollo: build_apollo_instance update_candid
	dfx build apollo --network ic && gzip -f -1 ./.dfx/ic/canisters/apollo/apollo.wasm
	dfx canister install --mode upgrade --wasm ./.dfx/ic/canisters/apollo/apollo.wasm.gz --network ic apollo