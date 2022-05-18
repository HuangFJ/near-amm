use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, ext_contract, log, near_bindgen, require, AccountId, Balance, PanicOnDefault};

// const GAS_FOR_TOKEN_TRANSFER: Gas = Gas(7_500_000_000_000);
// const BASE_GAS: Gas = Gas(20_000_000_000_000);
// const NO_GAS: Gas = Gas(0);
// const NO_DEPOSIT: u128 = 0;
const A_TICKER: u128 = 40000000000000000000000;
const B_TICKER: u128 = 300000000000000000000;

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
        a_ticker_after: Balance,
        b_ticker_after: Balance,
        contract_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
    );
    fn callback_update_tickers(&mut self, a_ticker_after: Balance, b_ticker_after: Balance);
}

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    owner_id: AccountId,
    ratio: u128,
    //total A token number
    a_ticker: Balance,
    a_contract_id: AccountId,
    a_contract_name: String,
    a_contract_decimals: u8,
    //total B token number
    b_ticker: Balance,
    b_contract_id: AccountId,
    b_contract_name: String,
    b_contract_decimals: u8,
}

#[near_bindgen]
impl Contract {
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
        //call A contract to get it's name and decimals
        ext_token::ext(this.a_contract_id.clone()).get_info().then(
            ext_self::ext(env::current_account_id()).callback_get_info(this.a_contract_id.clone()),
        );
        ext_token::ext(this.b_contract_id.clone()).get_info().then(
            ext_self::ext(env::current_account_id()).callback_get_info(this.b_contract_id.clone()),
        );
        //register this contract
        ext_token::ext(this.a_contract_id.clone()).register_amm(owner_id.clone(), this.a_ticker);
        ext_token::ext(this.b_contract_id.clone()).register_amm(owner_id, this.b_ticker);
        this
    }

    pub fn callback_get_info(&mut self, contract_id: AccountId, #[callback] val: (String, u8)) {
        require!(env::predecessor_account_id()==env::current_account_id(), "only support in self");
        log!("Fill additional info for {}", val.0);
        if contract_id == self.a_contract_id {
            self.a_contract_name = val.0;
            self.a_contract_decimals = val.1;
        } else if contract_id == self.b_contract_id {
            self.b_contract_name = val.0;
            self.b_contract_decimals = val.1;
        }
        self.calc_ratio();
    }

    pub fn get_info(
        &self,
    ) -> (
        (AccountId, String, Balance, u8),
        (AccountId, String, Balance, u8),
    ) {
        (
            (
                self.a_contract_id.clone(),
                self.a_contract_name.clone(),
                self.a_ticker,
                self.a_contract_decimals,
            ),
            (
                self.b_contract_id.clone(),
                self.b_contract_name.clone(),
                self.b_ticker,
                self.b_contract_decimals,
            ),
        )
    }

    pub fn get_ratio(&self) -> u128 {
        self.ratio
    }

    fn calc_ratio(&mut self) {
        let a_num = self.a_ticker / 10_u128.pow(self.a_contract_decimals as u32);
        let b_num = self.b_ticker / 10_u128.pow(self.b_contract_decimals as u32);
        //X * Y = K , K is some constant value
        self.ratio = a_num * b_num;
    }

    //deposit A , get B
    #[payable]
    pub fn deposit_a(&mut self, amount: Balance) {
        let sender_id = env::predecessor_account_id();
        let a_amount = amount * 10_u128.pow(self.a_contract_decimals as u32);
        let a_ticker_after = a_amount + self.a_ticker;
        let b_ticker_after = self.ratio / a_ticker_after;
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

    //deposit B , get A
    #[payable]
    pub fn deposit_b(&mut self, amount: Balance) {
        let sender_id = env::predecessor_account_id();
        let b_amount = amount * 10_u128.pow(self.b_contract_decimals as u32);
        let b_ticker_after = b_amount + self.b_ticker;
        let a_ticker_after = self.ratio / b_ticker_after;
        let a_amount = self.a_ticker - a_ticker_after;
        let next_contract = self.a_contract_id.clone();
        ext_token::ext(self.b_contract_id.clone())
            .transfer_from(sender_id.clone(), env::current_account_id(), b_amount)
            .then(
                ext_self::ext(env::current_account_id()).callback_ft_deposit(
                    a_ticker_after,
                    b_ticker_after,
                    next_contract,
                    sender_id,
                    a_amount,
                ),
            );
    }

    pub fn callback_ft_deposit(
        &mut self,
        a_ticker_after: Balance,
        b_ticker_after: Balance,
        contract_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
    ) {
        require!(env::predecessor_account_id()==env::current_account_id(), "only support in self");
        ext_token::ext(contract_id)
            .transfer_from(env::current_account_id(), receiver_id, amount)
            .then(
                ext_self::ext(env::current_account_id())
                    .callback_update_tickers(a_ticker_after, b_ticker_after),
            );
    }

    pub fn callback_update_tickers(&mut self, a_ticker_after: Balance, b_ticker_after: Balance) {
        require!(env::predecessor_account_id()==env::current_account_id(), "only support in self");
        self.a_ticker = a_ticker_after;
        self.b_ticker = b_ticker_after;
    }

}
