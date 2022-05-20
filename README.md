## Abstract
This a NEAR Smart Contract case named hello-near. Please notice that it's not ready for production.In this case we will create three simple NEAR contracts: A fungible token contract, B fungible token contract and AMM like contract.

The user can transfer a number of tokens A to the AMM contract and in return receive a certain number of tokens B (similarly in the other direction).The contract supports a certain ratio of tokens A and B. $X * Y = K$ ($K$ is some constant value, $X$ and $Y$ are the number of tokens A and B respectively)

## Create Accounts
NEAR is very different from Ethereum. It use Account system instead of the public key. One can own multiple accounts.The Account is like domain thing. You can create a top account which called `Master Account`, and then create multiple subaccounts under it.Because each NEAR account can only hold 1 smart contract. So in order to organize multiple contracts we can create "subaccounts" whose "master account" is user account.

In this example we need four accounts say `jonhuang.testnet`, `a.jonhuang.testnet`, `b.jonhuang.testnet`, `z.jonhuang.testnet`. `jonhuang.testnet` is the Master Account which I use to create subaccounts, deploy contract, manage contract. `a.jonhuang.testnet`, `b.jonhuang.testnet` and `z.jonhuang.testnet` are for A fungible token contract, B fungible token contract, AMM contract.

```bash
owner_id=jonhuang.testnet
a_id=a.jonhuang.testnet
b_id=b.jonhuang.testnet
amm_id=z.jonhuang.testnet

near login
near create-account $a_id --masterAccount $owner_id
near create-account $b_id --masterAccount $owner_id
near create-account $amm_id --masterAccount $owner_id
```
We use NEAR CLI to create the accounts. First we need to login with `jonhuang.testnet`. `jonhuang.testnet` was created via NEAR wallet app https://wallet.testnet.near.org/. And then create three contract accounts `a.jonhuang.testnet`, `b.jonhuang.testnet` and `amm.jonhuang.testnet`. All of them's master key is: `jonhuang.testnet`.

## Deploy and Init Smart Contract
```bash
near deploy $a_id --wasmFile="./token_contract.wasm"
near deploy $b_id --wasmFile="./token_contract.wasm"
near deploy $amm_id --wasmFile="./amm_contract.wasm"

near call $a_id new '{"owner_id":"'$owner_id'", "name":"A Token Contract", "symbol":"A", "total_supply":1000000000000, "decimals": 18}' --accountId=$owner_id
near call $b_id new '{"owner_id":"'$owner_id'", "name":"B Token Contract", "symbol":"B", "total_supply":20000000000000, "decimals": 15}' --accountId=$owner_id
near call $amm_id new '{"owner_id":"'$owner_id'", "a_contract_id":"'$a_id'", "b_contract_id":"'$b_id'"}' --accountId=$owner_id --gas=55000000000000
```
After deploying contracts, we use `near call` command to initialize them. Now we have all of three contracts live on the NEAR blockchain. A token contract has a total supply of 1,000,000,000,000 with decimals 18 which means $1,000,000,000,000 * 10^{18}$ minimum unit. B token contract has a total supply of 20,000,000,000,000 with decimals 15 which means $20,000,000,000,000 * 10^{15}$ minimum unit. Looking into the source code:
```rust
    pub fn new(
        owner_id: AccountId,
        name: String,
        symbol: String,
        total_supply: Balance,
        decimals: u8,
    ) -> Self {
        require!(!env::state_exists(), "The contract has initialized!");
        let mut this = Self {
            amm_id: None,
            token: FungibleToken::new(StorageKey::T),
            meta: FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name,
                symbol,
                icon: None,
                reference: None,
                reference_hash: None,
                decimals,
            },
        };
        // register owner account and set all initial tokens to him
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply * 10_u128.pow(decimals as u32));
        this
    }
```
And AMM contract source code:
```rust
    /// Initialization method:
    /// Input are the address of the contract owner and the addresses of two tokens (hereinafter token A and token B).
    /// requests and stores the metadata of tokens (name, decimals) and
    /// Creates wallets for tokens А & В.
    #[init]
    pub fn new(owner_id: AccountId, a_contract_id: AccountId, b_contract_id: AccountId) -> Self {
        require!(!env::state_exists(), "The contract has been initialized");

        let this = Self {
            owner_id: owner_id.clone(),
            ratio: 0,
            a_ticker: A_TICKER,
            a_contract_id,
            a_contract_name: "".into(),
            a_contract_decimals: 1,
            b_ticker: B_TICKER,
            b_contract_id,
            b_contract_name: "".into(),
            b_contract_decimals: 1,
        };
        // The method requests and stores the metadata of tokens (name, decimals)
        ext_token::ext(this.a_contract_id.clone()).get_info().then(
            ext_self::ext(env::current_account_id()).callback_get_info(this.a_contract_id.clone()),
        );
        ext_token::ext(this.b_contract_id.clone()).get_info().then(
            ext_self::ext(env::current_account_id()).callback_get_info(this.b_contract_id.clone()),
        );
        // Creates wallets for tokens А & В.
        ext_token::ext(this.a_contract_id.clone()).register_amm(owner_id.clone(), this.a_ticker);
        ext_token::ext(this.b_contract_id.clone()).register_amm(owner_id, this.b_ticker);
        this
    }
```
From above code, we saw sth like `ext_token::ext(this.a_contract_id.clone()).get_info()` which is a cross contract calling method. The statement means we call the `get_info` method of A contract from current contract and return a `Promise`. Therefore it is a asynchronous calling. So the current process do not block to wait the calling result, it continues runing to the end. The code is very simple. For ordinary, we should check every situation for the synchronous result. 

## Test AMM Functionality
```base
sim_id=b.jonhuang.testnet

near call $a_id storage_deposit '{"account_id": "'$sim_id'"}' --accountId=$owner_id --deposit=1
near call $a_id ft_transfer '{"receiver_id": "'$sim_id'","amount":"1000000000000000000000"}' --accountId=$owner_id --deposit=0.000000000000000000000001
near view $a_id ft_balance_of '{"account_id": "'$sim_id'"}'
```
First we use `b.jonhuang.testnet` account to simulate an AMM user. We register a A wallet for him and give him 1,000 tokens.
```bash
near call $amm_id deposit_a '{"amount":111}' --accountId=$sim_id --gas=55000000000000
```
This is a core function of AMM contract. We send a `deposit_a` transaction to AMM contract with the account of simulation user to tell AMM contract to exchange our 111 A tokens to a certain number of B tokens. Here the code is a little complicated.
```rust
    /// The user can transfer a certain number of tokens A to the contract account and 
    /// in return must receive a certain number of tokens B (similarly in the other direction).
    /// The contract supports a certain ratio of tokens A and B. X * Y = K 
    /// K is some constant value, X and Y are the number of tokens A and B respectively.
    #[payable]
    pub fn deposit_a(&mut self, amount: Balance) {
        let sender_id = env::predecessor_account_id();
        let decimal = 10_u128.pow(self.a_contract_decimals as u32);
        let a_amount = amount * decimal;
        let a_ticker_after = a_amount + self.a_ticker;
        let b_ticker_after = self.ratio
            / (a_ticker_after / decimal)
            * 10_u128.pow(self.b_contract_decimals as u32);
        let b_amount = self.b_ticker - b_ticker_after;
        let next_contract = self.b_contract_id.clone();
        ext_token::ext(self.a_contract_id.clone())
            .transfer_from(sender_id.clone(), env::current_account_id(), a_amount)
            .then(
                ext_self::ext(env::current_account_id()).callback_ft_deposit(
                    a_ticker_after,
                    b_ticker_after,
                    next_contract,
                    sender_id,
                    b_amount,
                ),
            );
    }
``` 
We first need to calculate the amount of B tokens before paying. $X * Y = K$ , $K$ is some constant value, $X$ and $Y$ are the number of tokens A and B respectively. When the user deposit a certain amount A say 111, the formula changes $(X + 111) * Y' = K$ . Then we got $Y' = K / (X + 111)$ . So we need pay him $Y - Y'$ of B tokens. We use below command to check both balances to make sure the exchange goes correctly. 
Since this is a synchronous transaction. So normally we should make rollback for all panics in there and commit while no incident occuring.

```bash
near view $a_id ft_balance_of '{"account_id": "'$sim_id'"}'
near view $b_id ft_balance_of '{"account_id": "'$sim_id'"}'
near view $a_id ft_balance_of '{"account_id": "'$amm_id'"}'
near view $b_id ft_balance_of '{"account_id": "'$amm_id'"}'
near view $amm_id get_info
near view $amm_id get_ratio
```

Finally we test the second core function, the owner of the contract can transfer a certain amount of tokens A or B to the contract account, thereby changing the ratio K.
```bash
near call $amm_id deposit_b_by_owner '{"amount":34321}' --accountId=$owner_id --gas=55000000000000
near view $amm_id get_info
near view $amm_id get_ratio
```


