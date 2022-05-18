use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{env, near_bindgen, AccountId, Gas};

// const TOKEN_TRANSFER_GAS: Gas=10000000;

#[near_bindgen]
pub struct Contract{
    owner_id: AccountId,
    a_id: AccountId,
    a_meta: FungibleTokenMetadata,
    b_id: AccountId,
    b_meta: FungibleTokenMetadata
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, a_contract_id: AccountId, b_contract_id: AccountId){
        
        // requests and stores the metadata of tokens

        // creates wallets for token A & B
        
        // let account_id = env::current_account_id();
        // let prepaid_gas = env::prepaid_gas() - TOKEN_TRANSFER_GAS;
        // let promise0 = env::promise_create(
        //     account_id.clone(),
        //     "factorial",
        //     &serde_json::to_vec(&(n - 1,)).unwrap(),
        //     0,
        //     prepaid_gas - FACTORIAL_MULT_CALL_GAS,
        // );
    }

    
}

