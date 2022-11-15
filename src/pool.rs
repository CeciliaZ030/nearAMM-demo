use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, Promise, require};
use near_sdk::collections::{UnorderedMap};
use near_sdk::serde::Serialize;
use near_sdk::json_types::U128;
use near_sdk::store::{LookupSet, UnorderedSet};

const ERR_BALANCE_OVERFLOW: &str = "Balance overflow";

#[derive(BorshDeserialize, BorshSerialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Pool {
    pub tokenA: AccountId,
    pub tokenB: AccountId,
    pub balanceA: Balance,
    pub balanceB: Balance,
}

impl Pool {

    pub fn new(tokenA: &AccountId, tokenB: &AccountId) -> Self {
        Self {
            tokenA: tokenA.clone(),
            tokenB: tokenB.clone(),
            balanceA: 0,
            balanceB: 0,
        }
    }

    pub fn incr_a(&mut self, amountA: U128) {
        self.balanceA
            .checked_add(amountA.into())
            .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
    }
    pub fn incr_b(&mut self, amountB: U128) {
        self.balanceB
            .checked_add(amountB.into())
            .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
    }
    pub fn decr_a(&mut self, amountA: U128) {
        self.balanceA
            .checked_sub(amountA.into())
            .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
    }
    pub fn decr_b(&mut self, amountB: U128) {
        self.balanceB
            .checked_sub(amountB.into())
            .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
    }

    pub fn incr(&mut self, token: &AccountId, amount: U128) {
        if token == &self.tokenA {
            self.balanceA
                .checked_add(amount.into())
                .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));

        } else {
            self.balanceB
                .checked_add(amount.into())
                .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
        }
    }

    pub fn decr(&mut self, token: &AccountId, amount: U128) {
        if token == &self.tokenA {
            self.balanceA
                .checked_sub(amount.into())
                .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));

        } else {
            self.balanceB
                .checked_sub(amount.into())
                .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
        }
    }

    pub fn calculate_in_to_out(&self, inToken: &AccountId, amount: U128) -> U128 {
        let constK = self.balanceA
            .checked_mul(self.balanceB)
            .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));

        if inToken == &self.tokenA {
            let newB = constK
                .checked_div(
                    self.balanceA.checked_add(Balance::from(amount))
                        .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW))
                ).unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
            U128::from(self.balanceB - newB)
        } else {
            let newA = constK
                .checked_div(
                    self.balanceB.checked_add(Balance::from(amount))
                        .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW))
                ).unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
            U128::from(self.balanceA - newA)
        }
    }
    pub fn calculate_out_to_in(&self, outToken: &AccountId, amount: U128) -> U128 {
        let constK = self.balanceA
            .checked_mul(self.balanceB)
            .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));

        if outToken == &self.tokenA {
            let newB = constK
                .checked_div(
                    self.balanceA.checked_sub(Balance::from(amount))
                        .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW))
                ).unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
            U128::from( newB - self.balanceB)
        } else {
            let newA = constK
                .checked_div(
                    self.balanceB.checked_sub(Balance::from(amount))
                        .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW))
                ).unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
            U128::from( newA - self.balanceA)
        }
    }

    pub fn calculate_a_to_b(&self, amountA: U128) -> U128 {
        let constK = self.balanceA
            .checked_mul(self.balanceB)
            .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
        let newB = constK
            .checked_div(
                self.balanceA.checked_add(Balance::from(amountA))
                        .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW))
            ).unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
        U128::from( self.balanceB - newB)
    }
    pub fn calculate_b_to_a(&self, amountB: U128) -> U128 {
        let constK = self.balanceA
            .checked_mul(self.balanceB)
            .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
        let newA = constK
            .checked_div(
                self.balanceB.checked_add(Balance::from(amountB))
                    .unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW))
            ).unwrap_or_else(|| env::panic_str(ERR_BALANCE_OVERFLOW));
        U128::from(self.balanceA- newA)
    }
}

