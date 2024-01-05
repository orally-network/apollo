all: local_deploy_apollo

update_candid:
	cargo test update_candid
	dfx generate 

build_apollo_instance:
	dfx build --check apollo_instance
	mv ./.dfx/local/canisters/apollo_instance/apollo_instance.wasm .


local_deploy_apollo: build_apollo_instance update_candid  
	dfx canister create apollo && dfx build apollo && gzip -f -1 ./.dfx/local/canisters/apollo/apollo.wasm
	dfx canister install --wasm ./.dfx/local/canisters/apollo/apollo.wasm.gz --argument \
		"(0:nat, \"dfx_test_key\", 10:nat)" apollo



local_upgrade: local_upgrade_apollo 

local_upgrade_apollo: build_apollo_instance update_candid 
	dfx build apollo 
	gzip -f -1 ./.dfx/local/canisters/apollo/apollo.wasm
	dfx canister install --mode upgrade --wasm ./.dfx/local/canisters/apollo/apollo.wasm.gz apollo


ic_upgrade: ic_upgrade_apollo


ic_upgrade_apollo: build_apollo_instance update_candid
	dfx build apollo --network ic && gzip -f -1 ./.dfx/ic/canisters/apollo/apollo.wasm
	dfx canister install --mode upgrade --wasm ./.dfx/ic/canisters/apollo/apollo.wasm.gz --network ic apollo