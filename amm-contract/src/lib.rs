use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, ext_contract, log, near_bindgen, require, AccountId, Balance, PanicOnDefault};

const A_BALANCE: u128 = 40_000_000_000_000_000_000_000;
const B_BALANCE: u128 =    300_000_000_000_000_000_000;

#[ext_contract(ext_token)]
trait ExtToken {
    fn get_info(&self) -> (String, u8);
    fn register_amm(&mut self, sender_id: AccountId, amount: Balance);
    fn transfer_from(&mut self, sender_id: AccountId, receiver_id: AccountId, amount: Balance);
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn callback_get_info(&mut self, contract_id: AccountId, #[callback] val: (String, u8));
    fn callback_ft_deposit(
        &mut self,
        a_balance_after: Balance,
        b_balance_after: Balance,
        contract_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
    );
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
    /// Initialization method:
    /// Input are the address of the contract owner and the addresses of two tokens (hereinafter token A and token B).
    /// requests and stores the metadata of tokens (name, decimals) and
    /// Creates wallets for tokens А & В.
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
        // Creates wallets for tokens А & В.
        ext_token::ext(a_contract_id.clone()).register_amm(owner_id.clone(), A_BALANCE);
        ext_token::ext(b_contract_id.clone()).register_amm(owner_id.clone(), B_BALANCE);

        Self {
            owner_id,
            ratio: 0,
            a_balance: A_BALANCE,
            a_meta: TokenMeta {
                account_id: a_contract_id,
                ticker: "".into(),
                decimal: 1,
            },
            b_balance: B_BALANCE,
            b_meta: TokenMeta {
                account_id: b_contract_id,
                ticker: "".into(),
                decimal: 1,
            },
        }
    }

    pub fn callback_get_info(&mut self, contract_id: AccountId, #[callback] val: (String, u8)) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "only support in self"
        );
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

    fn calc_ratio(&mut self) {
        let a_num = self.a_balance / 10_u128.pow(self.a_meta.decimal);
        let b_num = self.b_balance / 10_u128.pow(self.b_meta.decimal);
        //X * Y = K , K is some constant value
        self.ratio = a_num * b_num;
    }

    /// The user can transfer a certain number of tokens A to the contract account and
    /// in return must receive a certain number of tokens B (similarly in the other direction).
    /// The contract supports a certain ratio of tokens A and B. X * Y = K
    /// K is some constant value, X and Y are the number of tokens A and B respectively.
    #[payable]
    pub fn deposit_a(&mut self, amount: Balance) {
        let sender_id = env::predecessor_account_id();
        let decimal = 10_u128.pow(self.a_meta.decimal);
        let a_amount = amount * decimal;
        let a_balance_after = a_amount + self.a_balance;
        let b_balance_after =
            self.ratio / (a_balance_after / decimal) * 10_u128.pow(self.b_meta.decimal);
        let b_amount = self.b_balance - b_balance_after;
        let next_contract = self.b_meta.account_id.clone();
        ext_token::ext(self.a_meta.account_id.clone())
            .transfer_from(sender_id.clone(), env::current_account_id(), a_amount)
            .then(
                ext_self::ext(env::current_account_id()).callback_ft_deposit(
                    a_balance_after,
                    b_balance_after,
                    next_contract,
                    sender_id,
                    b_amount,
                ),
            );
    }

    /// The owner of the contract can transfer a certain amount of tokens A or B to the contract account,
    /// thereby changing the ratio K.
    #[payable]
    pub fn deposit_a_by_owner(&mut self, amount: Balance) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "only support to call by itself"
        );
        let a_amount = amount * 10_u128.pow(self.a_meta.decimal);
        let a_balance_after = a_amount + self.a_balance;
        let b_balance_after = self.b_balance;
        ext_token::ext(self.a_meta.account_id.clone())
            .transfer_from(self.owner_id.clone(), env::current_account_id(), a_amount)
            .then(
                ext_self::ext(env::current_account_id())
                    .callback_update_balances(a_balance_after, b_balance_after),
            );
    }

    /// in the opposite direction
    #[payable]
    pub fn deposit_b(&mut self, amount: Balance) {
        let sender_id = env::predecessor_account_id();
        let decimal = 10_u128.pow(self.b_meta.decimal);
        let b_amount = amount * decimal;
        let b_balance_after = b_amount + self.b_balance;
        let a_balance_after =
            self.ratio / (b_balance_after / decimal) * 10_u128.pow(self.a_meta.decimal);
        let a_amount = self.a_balance - a_balance_after;
        let next_contract = self.a_meta.account_id.clone();
        ext_token::ext(self.b_meta.account_id.clone())
            .transfer_from(sender_id.clone(), env::current_account_id(), b_amount)
            .then(
                ext_self::ext(env::current_account_id()).callback_ft_deposit(
                    a_balance_after,
                    b_balance_after,
                    next_contract,
                    sender_id,
                    a_amount,
                ),
            );
    }

    #[payable]
    pub fn deposit_b_by_owner(&mut self, amount: Balance) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "only support to call by itself"
        );
        let b_amount = amount * 10_u128.pow(self.b_meta.decimal);
        let b_balance_after = b_amount + self.b_balance;
        let a_balance_after = self.a_balance;
        ext_token::ext(self.b_meta.account_id.clone())
            .transfer_from(self.owner_id.clone(), env::current_account_id(), b_amount)
            .then(
                ext_self::ext(env::current_account_id())
                    .callback_update_balances(a_balance_after, b_balance_after),
            );
    }

    pub fn callback_ft_deposit(
        &mut self,
        a_balance_after: Balance,
        b_balance_after: Balance,
        contract_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
    ) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "only support to call by itself"
        );
        ext_token::ext(contract_id)
            .transfer_from(env::current_account_id(), receiver_id, amount)
            .then(
                ext_self::ext(env::current_account_id())
                    .callback_update_balances(a_balance_after, b_balance_after),
            );
    }

    pub fn callback_update_balances(&mut self, a_balance_after: Balance, b_balance_after: Balance) {
        require!(
            env::predecessor_account_id() == env::current_account_id(),
            "only support to call by itself"
        );
        self.a_balance = a_balance_after;
        self.b_balance = b_balance_after;
        self.calc_ratio();
    }
}
