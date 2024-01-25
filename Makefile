all: local_deploy_apollo

update_candid:
	cargo test update_candid
	dfx generate 

build_apollo_instance:
	dfx build --check apollo_instance
	mv ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm .


local_deploy_apollo: update_candid build_apollo_instance   
	dfx canister create apollo && dfx build apollo && gzip -f -1 ./.dfx/local/canisters/apollo/apollo.wasm
	dfx canister install --wasm ./.dfx/local/canisters/apollo/apollo.wasm.gz --argument \
		"(0:nat, \"dfx_test_key\", 10:nat)" apollo

local_deploy_apollo_instance: update_candid 
	dfx canister create apollo_instance && dfx build apollo_instance && gzip -f -1 ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm
	dfx canister install --wasm ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm.gz --argument \
		"(record {\
			apollos_fee = 0:nat; \
			key_name = \"dfx_test_key\"; \
			chain_id = 17000:nat; \
			chain_rpc = \"https://l1rpc.katla.taiko.xyz\"; \
			apollo_coordinator = \"0xC1e42d86716f8b8fA616249112a21622b07319a3\"; \
			timer_frequency = 30:nat64; \
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