owner_id=jonhuang.testnet
a_id=a.jonhuang.testnet
b_id=b.jonhuang.testnet
amm_id=z.jonhuang.testnet

near delete $a_id $owner_id
near delete $b_id $owner_id
near delete $amm_id $owner_id
near create-account $a_id --masterAccount $owner_id
near create-account $b_id --masterAccount $owner_id
near create-account $amm_id --masterAccount $owner_id
near deploy $a_id --wasmFile="./token_contract.wasm"
near deploy $b_id --wasmFile="./token_contract.wasm"
near deploy $amm_id --wasmFile="./amm_contract.wasm"
near call $a_id new '{"owner_id":"'$owner_id'", "name":"A Token Contract", "symbol":"A", "total_supply":1000000000000, "decimals": 18}' --accountId=$owner_id
near call $b_id new '{"owner_id":"'$owner_id'", "name":"B Token Contract", "symbol":"B", "total_supply":20000000000000, "decimals": 15}' --accountId=$owner_id
near call $amm_id new '{"owner_id":"'$owner_id'", "a_contract_id":"'$a_id'", "b_contract_id":"'$b_id'"}' --accountId=$owner_id