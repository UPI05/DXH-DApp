use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, env, AccountId, PromiseError, Promise, Gas, Balance};
use json::{parse};
use serde_json::{Value, json, from_str};

const CALL_GAS: Gas = Gas(10_000_000_000_000);
const TOKEN_SC_ADDR: &str = "dev-1663407143254-90994928167650";
const VALIDATOR_ACCOUNT: &str = "upi05.testnet";
const POOL_AMOUNT_MIN: u128 = 10;

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    candidates: Vec<String>,
    verified_candidates: Vec<String>,
    pool_balance: u128
}

// Define the default, which automatically initializes the contract
impl Default for Contract {
    fn default() -> Self {
        Self {
            candidates: vec![],
            verified_candidates: vec![],
            pool_balance: 0
        }
    }
}

// Implement the contract structure
#[near_bindgen]
impl Contract {
    // pool balance
    #[private]
    pub fn get_pool_balance(&self) {
        let args = json!({
            "account_id": env::current_account_id()
        }) .to_string().into_bytes().to_vec();
        // why does we convert args to vector and use json here?

        let promise = Promise::new(TOKEN_SC_ADDR.parse().unwrap())
            .function_call("ft_balance_of".to_string(), args.clone(), 1, CALL_GAS);
            //why does CALL_GAS appear here and set to 10M? 
        promise.then(
            Promise::new(env::current_account_id())
            .function_call("get_pool_balance_callback".to_string(), args, 0, CALL_GAS)
        );
    }

    #[private]
    pub fn get_pool_balance_callback(&mut self, #[callback_result] call_result: Result<String, PromiseError>) {
        // Check if the promise succeeded by calling the method outlined in external.rs
        if call_result.is_err() {
          log!("There was an error while contacting NEAR");
        } else {
            let res: String = call_result.unwrap();
            self.pool_balance = res.parse().unwrap();
        }
    }

    // candidates
    pub fn set_candidate(&mut self, candidate: String) {
        parse(&candidate).expect("Wrong format!");
        // why use parse(candidate) substitute for cadidate.parse()?
        self.candidates.push(candidate);
    }
    
    pub fn get_candidates(&self) -> Vec<String> {
        self.candidates.clone()
    }

    pub fn remove_candidate(&mut self, candidate: String) {
        self.candidates.retain(|x| * x != candidate);
    }
    

    // verified candidates
    pub fn set_verified_candidate(&mut self, candidate: String, amount: String) {
        let account_id = env::signer_account_id();
        
        if account_id.to_string() == VALIDATOR_ACCOUNT.to_string() {
            // Set MAX_COIN before pushing
            let mut verfied_candidate: Value = serde_json::from_str(&candidate).unwrap();
            verfied_candidate["donatedAmount"] = json!(amount);
            self.verified_candidates.push(serde_json::to_string(&verfied_candidate).unwrap());
            self.remove_candidate(candidate);

        }
    }
    
    pub fn get_verified_candidates(&self) -> Vec<String> {
        self.verified_candidates.clone()
    }

    pub fn remove_verified_candidate(&mut self, candidate: String) {
        let account_id = env::signer_account_id();
        if account_id.to_string() == VALIDATOR_ACCOUNT.to_string() {
            self.verified_candidates.retain(|x| * x != candidate);
        }
    }

    // donate
    // Call this function to trigger token widthdraw process from donation pool
    #[payable]
    pub fn donate(&mut self) {
        if env::signer_account_id().to_string() == VALIDATOR_ACCOUNT.to_string() {
            
            let verified_candidates: Vec<String> = self.get_verified_candidates();
            
            for i in 0..verified_candidates.len() {
                self.get_pool_balance();

                let candidate: Value = serde_json::from_str(&verified_candidates[i]).unwrap();

                let donated_amount: u128 = candidate["donatedAmount"].to_string().replace("\"", "").parse().unwrap();

                if donated_amount + POOL_AMOUNT_MIN <= self.pool_balance {
                    let args = json!({
                        "receiver_id": candidate["publicKey"].to_string().replace("\"", ""),
                        "amount": candidate["donatedAmount"].to_string().replace("\"", "")
                    }).to_string().into_bytes().to_vec();
                    Promise::new(TOKEN_SC_ADDR.parse().unwrap())
                    .function_call("ft_transfer".to_string(), args, 1, CALL_GAS);

                    self.remove_verified_candidate(verified_candidates[i].clone());
                } else {
                    break;
                }
            }
        }
    }
}

// Unit test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_and_set_candidates_testing() {
        let mut contract = Contract::default();
        let candidate: String = r#"

        {
            "publicKey": "upi05.testnet"
        }
        
        "#.to_owned();
        contract.set_candidate(candidate.clone());
        assert_eq!(
            contract.get_candidates()[0],
            candidate
        );
    }
    
}
