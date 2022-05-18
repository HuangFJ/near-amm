use near_contract_standards;
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_contract_standards::fungible_token::FungibleToken;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Balance, BorshStorageKey, PanicOnDefault,
    PromiseOrValue,
};

// specify total supply
const TOTAL_SUPPLY: u128 = 100_000_000_000_000_000_000_000_000_000;


#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    meta: LazyOption<FungibleTokenMetadata>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    T,
    M,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        require!(!env::state_exists(), "The contract has initialized!");
        let metadata = FungibleTokenMetadata {
            spec: FT_METADATA_SPEC.to_string(),
            name: "A Token".to_string(),
            symbol: "A".to_string(),
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 18,
        };
        let mut this = Self {
            token: FungibleToken::new(StorageKey::T),
            meta: LazyOption::new(StorageKey::M, Some(&metadata)),
        };

        // this.token.internal_register_account(&owner_id);
        // this.token.internal_deposit(&owner_id, TOTAL_SUPPLY);
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);
