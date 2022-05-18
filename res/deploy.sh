owner_id=jonhuang.testnet
a_id=a.jonhuang.testnet
b_id=b.jonhuang.testnet
amm_id=z.jonhuang.testnet
sim_id=b.jonhuang.testnet

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
near call $amm_id new '{"owner_id":"'$owner_id'", "a_contract_id":"'$a_id'", "b_contract_id":"'$b_id'"}' --accountId=$owner_id --gas=55000000000000


near view $a_id ft_balance_of '{"account_id": "'$amm_id'"}'
near view $b_id ft_balance_of '{"account_id": "'$amm_id'"}'
near view $amm_id get_info
near view $amm_id get_ratio


near call $a_id storage_deposit '{"account_id": "'$sim_id'"}' --accountId=$owner_id --deposit=1
#1000
near call $a_id ft_transfer '{"receiver_id": "'$sim_id'","amount":"1000000000000000000000"}' --accountId=$owner_id --deposit=0.000000000000000000000001
near view $a_id ft_balance_of '{"account_id": "'$sim_id'"}'
near call $amm_id deposit_a '{"amount":111}' --accountId=$sim_id --gas=55000000000000
near view $a_id ft_balance_of '{"account_id": "'$sim_id'"}'
near view $b_id ft_balance_of '{"account_id": "'$sim_id'"}'
near view $a_id ft_balance_of '{"account_id": "'$amm_id'"}'
near view $b_id ft_balance_of '{"account_id": "'$amm_id'"}'
near view $amm_id get_info
near view $amm_id get_ratio

near call $amm_id deposit_b_by_owner '{"amount":34321}' --accountId=$owner_id --gas=55000000000000
near view $amm_id get_info
near view $amm_id get_ratio