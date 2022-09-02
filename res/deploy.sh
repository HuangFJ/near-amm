owner_id=geek.testnet
a_id=a.$owner_id
b_id=b.$owner_id
amm_id=amm.$owner_id
sim_id=sim.$owner_id

#40000
amm_a_balance=(40000000000000000000000)
#300000
amm_b_balance=(  300000000000000000000)
#1000
sim_a_balance=( 1000000000000000000000)
#111
sim_a_exchange=(    111000000000000000)

near="near --nodeUrl https://rpc.testnet.near.org"

$near delete $a_id $owner_id
$near delete $b_id $owner_id
$near delete $amm_id $owner_id
$near delete $sim_id $owner_id
$near create-account $a_id --masterAccount $owner_id
$near create-account $b_id --masterAccount $owner_id
$near create-account $amm_id --masterAccount $owner_id
$near create-account $sim_id --masterAccount $owner_id
$near deploy $a_id --wasmFile="./token_contract.wasm"
$near deploy $b_id --wasmFile="./token_contract.wasm"
$near deploy $amm_id --wasmFile="./amm_contract.wasm"

$near call $a_id new '{"owner_id":"'$owner_id'", "name":"A Token Contract", "symbol":"A", "total_supply":1000000000000, "decimals": 18}' --accountId=$owner_id
$near call $b_id new '{"owner_id":"'$owner_id'", "name":"B Token Contract", "symbol":"B", "total_supply":20000000000000, "decimals": 15}' --accountId=$owner_id
$near call $amm_id new '{"owner_id":"'$owner_id'", "a_contract_id":"'$a_id'", "b_contract_id":"'$b_id'"}' --accountId=$owner_id --gas=55000000000000

$near call $a_id storage_deposit '{"account_id": "'$amm_id'"}' --accountId=$owner_id --deposit=1
$near call $b_id storage_deposit '{"account_id": "'$amm_id'"}' --accountId=$owner_id --deposit=1
$near call $a_id storage_deposit '{"account_id": "'$sim_id'"}' --accountId=$owner_id --deposit=1
$near call $b_id storage_deposit '{"account_id": "'$sim_id'"}' --accountId=$owner_id --deposit=1

$near call $a_id ft_transfer_call '{"receiver_id": "'$amm_id'","amount":"'$amm_a_balance'","msg":""}' --accountId=$owner_id --deposit=0.000000000000000000000001 --gas=55000000000000
$near call $b_id ft_transfer_call '{"receiver_id": "'$amm_id'","amount":"'$amm_b_balance'","msg":""}' --accountId=$owner_id --deposit=0.000000000000000000000001 --gas=55000000000000
$near call $a_id ft_transfer '{"receiver_id": "'$sim_id'","amount":"'$sim_a_balance'"}' --accountId=$owner_id --deposit=0.000000000000000000000001 --gas=55000000000000

#1000
$near view $a_id ft_balance_of '{"account_id": "'$sim_id'"}'
#0
$near view $b_id ft_balance_of '{"account_id": "'$sim_id'"}'
#40000
$near view $a_id ft_balance_of '{"account_id": "'$amm_id'"}'
#300000
$near view $b_id ft_balance_of '{"account_id": "'$amm_id'"}'
$near view $amm_id get_info
#12000000000
$near view $amm_id get_ratio

$near call $a_id ft_transfer_call '{"receiver_id": "'$amm_id'","amount":"'$sim_a_exchange'","msg":""}' --accountId=$sim_id --deposit=0.000000000000000000000001 --gas=55000000000000

#1000-111=889
$near view $a_id ft_balance_of '{"account_id": "'$sim_id'"}'
#x
$near view $b_id ft_balance_of '{"account_id": "'$sim_id'"}'
#40000+111=40111
$near view $a_id ft_balance_of '{"account_id": "'$amm_id'"}'
#300000-x
$near view $b_id ft_balance_of '{"account_id": "'$amm_id'"}'
$near view $amm_id get_info
#12000000000
$near view $amm_id get_ratio
