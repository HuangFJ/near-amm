use near_contract_standards;
use near_contract_standards::fungible_token::metadata::{FungibleTokenMetadata, FT_METADATA_SPEC};
use near_contract_standards::fungible_token::FungibleToken;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{
    env, log, near_bindgen, require, AccountId, Balance, BorshStorageKey, PanicOnDefault,
    PromiseOrValue,
};

/// a fungible token template
/// we can deploy to multiple accounts and init with different parameters to create different tokens
/// there is a amm_id attribute which reffers to the AMM Contract
/// we use this attribute to authorize and verify transfer_from method
/// this is not a proper way when using in production
#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    meta: FungibleTokenMetadata,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    T,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        owner_id: AccountId,
        name: String,
        symbol: String,
        total_supply: Balance,
        decimals: u8,
    ) -> Self {
        require!(!env::state_exists(), "The contract has initialized!");
        let mut this = Self {
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

        this.token.internal_register_account(&owner_id);
        this.token
            .internal_deposit(&owner_id, total_supply * 10_u128.pow(decimals as u32));
        this
    }

    pub fn get_info(self) -> (String, u8) {
        (self.meta.symbol, self.meta.decimals)
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;
    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        let name = "A Token Contract";
        let symbol = "A";
        let total_supply = 10000000000000000000000;
        let decimals = 18;

        testing_env!(context.build());
        let contract = Contract::new(
            accounts(1).into(),
            name.into(),
            symbol.into(),
            total_supply,
            decimals,
        );
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, total_supply);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, total_supply);
        assert_eq!(contract.get_info(), (name.into(), decimals));
    }
}
