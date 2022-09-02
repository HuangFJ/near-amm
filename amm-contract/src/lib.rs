use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{
    assert_self, env, ext_contract, log, near_bindgen, require, AccountId, Balance, PanicOnDefault,
    Promise, PromiseOrValue, PromiseResult,
};

#[ext_contract(ext_token)]
trait ExtToken {
    fn get_info(&self) -> (String, u8);
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128);
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn callback_get_info(&mut self, contract_id: AccountId, #[callback] val: (String, u8));
    fn callback_update_balances(&mut self, a_balance_after: Balance, b_balance_after: Balance);
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct TokenMeta {
    account_id: AccountId,
    ticker: String,
    decimal: u32,
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner_id: AccountId,
    ratio: u128,
    //total A token number
    a_balance: u128,
    a_meta: TokenMeta,
    //total B token number
    b_balance: u128,
    b_meta: TokenMeta,
}

#[near_bindgen]
impl Contract {
    /// Input are the address of the contract owner and the addresses of two tokens (hereinafter token A and token B).
    /// requests and stores the metadata of tokens (name, decimals)
    #[init]
    pub fn new(owner_id: AccountId, a_contract_id: AccountId, b_contract_id: AccountId) -> Self {
        require!(!env::state_exists(), "The contract has been initialized");

        // The method requests and stores the metadata of tokens (name, decimals)
        ext_token::ext(a_contract_id.clone()).get_info().then(
            ext_self::ext(env::current_account_id()).callback_get_info(a_contract_id.clone()),
        );
        ext_token::ext(b_contract_id.clone()).get_info().then(
            ext_self::ext(env::current_account_id()).callback_get_info(b_contract_id.clone()),
        );

        Self {
            owner_id,
            ratio: 0,
            a_balance: 0,
            a_meta: TokenMeta {
                account_id: a_contract_id,
                ticker: "".into(),
                decimal: 1,
            },
            b_balance: 0,
            b_meta: TokenMeta {
                account_id: b_contract_id,
                ticker: "".into(),
                decimal: 1,
            },
        }
    }

    pub fn callback_update_balances(
        &mut self,
        a_balance_after: Balance,
        b_balance_after: Balance,
    ) -> PromiseOrValue<U128> {
        assert_self();

        match env::promise_result(0) {
            PromiseResult::NotReady => env::abort(),
            PromiseResult::Successful(_) => {
                self.update_balances(a_balance_after, b_balance_after);
                PromiseOrValue::Value(0.into())
            }
            PromiseResult::Failed => env::panic_str("fail!"),
        }
    }

    pub fn callback_get_info(&mut self, contract_id: AccountId, #[callback] val: (String, u8)) {
        assert_self();

        log!("Fill additional info for {}", val.0);
        if contract_id == self.a_meta.account_id {
            self.a_meta.ticker = val.0;
            self.a_meta.decimal = val.1 as u32;
        } else if contract_id == self.b_meta.account_id {
            self.b_meta.ticker = val.0;
            self.b_meta.decimal = val.1 as u32;
        }
        self.calc_ratio();
    }

    pub fn get_info(
        &self,
    ) -> (
        (AccountId, String, Balance, u32),
        (AccountId, String, Balance, u32),
    ) {
        (
            (
                self.a_meta.account_id.clone(),
                self.a_meta.ticker.clone(),
                self.a_balance,
                self.a_meta.decimal,
            ),
            (
                self.b_meta.account_id.clone(),
                self.b_meta.ticker.clone(),
                self.b_balance,
                self.b_meta.decimal,
            ),
        )
    }

    pub fn get_ratio(&self) -> u128 {
        self.ratio
    }

    pub fn calc_ratio(&mut self) {
        let a_num = self.a_balance / 10_u128.pow(self.a_meta.decimal);
        let b_num = self.b_balance / 10_u128.pow(self.b_meta.decimal);
        //X * Y = K , K is some constant value
        self.ratio = a_num * b_num;
    }

    fn deposit_a(&mut self, sender_id: AccountId, a_amount: Balance) -> Promise {
        let decimal = 10_u128.pow(self.a_meta.decimal);
        let a_balance_after = a_amount + self.a_balance;
        let b_balance_after = ((self.ratio * decimal / a_balance_after) as f64
            * 10_u128.pow(self.b_meta.decimal) as f64) as u128;
        let b_amount = self.b_balance - b_balance_after;
        let next_contract = self.b_meta.account_id.clone();

        ext_token::ext(next_contract)
            .with_attached_deposit(1)
            .ft_transfer(sender_id, b_amount.into())
            .then(
                ext_self::ext(env::current_account_id())
                    .callback_update_balances(a_balance_after, b_balance_after),
            )
    }

    fn deposit_a_by_owner(&mut self, a_amount: Balance) {
        let a_balance_after = a_amount + self.a_balance;
        let b_balance_after = self.b_balance;

        self.update_balances(a_balance_after, b_balance_after);
    }

    /// in the opposite direction
    fn deposit_b(&mut self, sender_id: AccountId, b_amount: Balance) -> Promise {
        let decimal = 10_u128.pow(self.b_meta.decimal);
        let b_balance_after = b_amount + self.b_balance;
        let a_balance_after = ((self.ratio * decimal / b_balance_after) as f64
            * 10_u128.pow(self.a_meta.decimal) as f64) as u128;
        let a_amount = self.a_balance - a_balance_after;
        let next_contract = self.a_meta.account_id.clone();

        ext_token::ext(next_contract)
            .with_attached_deposit(1)
            .ft_transfer(sender_id, a_amount.into())
            .then(
                ext_self::ext(env::current_account_id())
                    .callback_update_balances(a_balance_after, b_balance_after),
            )
    }

    fn deposit_b_by_owner(&mut self, b_amount: Balance) {
        let b_balance_after = b_amount + self.b_balance;
        let a_balance_after = self.a_balance;

        self.update_balances(a_balance_after, b_balance_after);
    }

    fn update_balances(&mut self, a_balance_after: Balance, b_balance_after: Balance) {
        self.a_balance = a_balance_after;
        self.b_balance = b_balance_after;
        self.calc_ratio();
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// The user can transfer a certain number of tokens A to the contract account and
    /// in return must receive a certain number of tokens B (similarly in the other direction).
    /// The contract supports a certain ratio of tokens A and B. X * Y = K
    /// K is some constant value, X and Y are the number of tokens A and B respectively.
    /// The owner of the contract can transfer a certain amount of tokens A or B to the contract account,
    /// thereby changing the ratio K.
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let _ = msg;
        let predecessor_id = env::predecessor_account_id();
        if predecessor_id == self.a_meta.account_id && sender_id != self.owner_id {
            self.deposit_a(sender_id, amount.into()).into()
        } else if predecessor_id == self.a_meta.account_id && sender_id == self.owner_id {
            self.deposit_a_by_owner(amount.into());
            PromiseOrValue::Value(0.into())
        } else if predecessor_id == self.b_meta.account_id && sender_id != self.owner_id {
            self.deposit_b(sender_id, amount.into()).into()
        } else if predecessor_id == self.b_meta.account_id && sender_id == self.owner_id {
            self.deposit_b_by_owner(amount.into());
            PromiseOrValue::Value(0.into())
        } else {
            env::panic_str("invalid call");
        }
    }
}
