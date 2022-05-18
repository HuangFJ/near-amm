Today we gona crreate a near case named hello-near.
in this case we create three contracts : a-token, b-token and amm-contract
a-token and b-token are fungible token contract
Each NEAR account can only hold 1 smart contract. 
For applications where users should be able to organize multiple contracts you can create "subaccounts" whose "master account" is the user account.

we need create 2 accounts say: jonhuang.testnet, jonhuang_0000.testnet
then transfer all NEARs from  jonhuang_0000.testnet to jonhuang.testnet
jonhuang.testnet account now have 400 NEARs

use near cli to login with jonhuang.testnet
create three contract account a.jonhuang.testnet, b.jonhuang.testnet and amm.jonhuang.testnet
all of them's master key is: jonhuang.testnet

delete a.jonhuang.testnet, transfer remaining funds to jonhuang.testnet
>near delete a.jonhuang.testnet jonhuang.testnet

--masterAccount is Master account used when creating new accounts
>near create-account a.jonhuang.testnet --masterAccount jonhuang.testnet
>near create-account a.a.jonhuang.testnet --masterAccount a.jonhuang.testnet

--accountId is Account name of contract
>near deploy --wasmFile a_token.wasm --accountId a.jonhuang.testnet

we are not signing a transaction, didn't include an --accountId flag
>near view a.jonhuang.testnet ft_total_supply

--accountId specify a NEAR account that will sign it, using the credentials files we looked at.
>near call a.jonhuang.testnet new --accountId jonhuang.testnet
>near call a.jonhuang.testnet func '{"arg":"par"}' --accountId jonhuang.testnet


The id of the account that owns the current contract.
>env::current_account_id()

The id of the account that either signed the original transaction or issued the initial cross-contract call.
>env::signer_account_id()
The public key of the account that did the signing.
>env::signer_account_pk()

The id of the account that was the previous contract in the chain of cross-contract calls.
If this is the first contract, it is equal to `signer_account_id`.
>env::predecessor_account_id()

Reset the account's contract and state
>near delete a.jonhuang.testnet jonhuang.testnet
>near create-account a.jonhuang.testnet --masterAccount jonhuang.testnet

NEAR uses human readable account IDs instead of a public key hash as the account identifier and many keys (public/private key pairs) can be created for each account that we call "Access Keys". Currently, there are two types of access keys; 'FullAccess' & 'FunctionCall'.
1) Create Account 2) Delete Account 3) Add Key 4) Delete Key 5) Deploy Contract 6) Function Call 7) Transfer Ⓝ 8) Stake Ⓝ (for validators)

用于调用不需要支付的合约方法
A FunctionCall key is unique as it only has permission to call a smart contract's method(s) that do not attach Ⓝ as a deposit (i.e. payable functions). These keys have the following three attributes:

1) allowance - the amount of Ⓝ the key is allowed to spend on gas fees (optional - default: null) 2) receiver_id - contract the key is allowed to call methods on (required) 3) method_names - contract methods the key is allowed to call (optional) 

