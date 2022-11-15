use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, require, PromiseOrValue, Promise};
use near_sdk::collections::{UnorderedMap};
use near_sdk::serde::Serialize;
use near_sdk::json_types::{U128, ValidAccountId};
use near_sdk::store::{LookupSet, key::Keccak256, LookupMap};

use near_contract_standards::fungible_token::{
    metadata::FungibleTokenMetadata,
    receiver::FungibleTokenReceiver,
};

use std::str::FromStr;
use std::convert::TryInto;
use near_contract_standards::non_fungible_token::metadata::TokenMetadata;
use near_sdk::env::{abort, panic_str};
use pool::Pool;
use ft_token::*;

mod ft_token;
mod pool;

const INITIAL_BALANCE: Balance = 250_000_000_000_000_000_000_000; // 2.5e23yN, 0.25N


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Market {
    pub owner: AccountId,
    // (tokenA, tokenB) -> pool
    pub pools: LookupMap<(AccountId, AccountId), Pool>,
    // (user, token) -> Balance
    pub user_reserves: LookupMap<(AccountId, AccountId), Balance>,
}

#[near_bindgen]
impl Market {
    #[init]
    #[private] // Public - but only callable by env::current_account_id()
    pub fn init(owner: AccountId) -> Self {
        Self {
            owner,
            pools: LookupMap::new(b"d"),
            user_reserves: LookupMap::new(b"u"),
        }
    }

    #[private]
    pub fn change_owner(&mut self, owner: AccountId) {
        self.owner = owner;
    }

    pub fn init_pool(&mut self, tokenA: AccountId, tokenB: AccountId) {
        let pool = pool::Pool::new(&tokenA, &tokenB);
        self.pools.insert((tokenA, tokenB), pool);
    }

    pub fn get_pool(&self, tokenIn: AccountId, tokenOut: AccountId) -> Option<&Pool> {
        match self.pools.get(&(tokenIn.clone(), tokenOut.clone())) {
            Some(p) => Some(p),
            None => self.pools.get(&(tokenOut.clone(), tokenIn.clone()))
        }
    }

    pub fn get_token_metadata(&self, token: AccountId) -> Promise {
        ext_ft_metadata::ext(token.clone()).ft_metadata()
    }

    pub fn get_token_liquidity(&self, token: AccountId) -> Promise {
        ext_fungible_token::ext(token.clone())
            .ft_balance_of(env::current_account_id())
    }

    /*
    Paying inA amount of A in exchange of minB amount of B.
    If outbounding amount of B is not enough, revert.
    */
    pub fn swap_min(
        &mut self,
        tokenIn: AccountId,
        amountIn: U128,
        tokenOut: AccountId,
        min: U128
    ){
        let pool = self.get_pool(tokenIn.clone(), tokenOut.clone())
            .unwrap_or_else(|| env::panic_str("Pool does not exist."));;
        let user = env::predecessor_account_id();
        match self.user_reserves.get(&(user.clone(), tokenIn.clone())) {
            Some(balance) => require!(balance > &amountIn.into()),
            None => panic_str("User does not exist.")
        }

        let amountOut = pool.calculate_in_to_out(&tokenIn, amountIn);
        require!(amountOut >= min);

        self._swap(&user, &tokenIn, amountIn, &tokenOut, amountOut);
    }
    /*
    Paying maxA amount of A in exchange of outB amount of B.
    If amount of A needed is to large, revert.
    */
    pub fn swap_max(
        &mut self,
        tokenIn: AccountId,
        maxIn: U128,
        tokenOut: AccountId,
        amountOut: U128
    ) {
        let pool = self.get_pool(tokenIn.clone(), tokenOut.clone())
            .unwrap_or_else(|| env::panic_str("Pool does not exist."));;
        let user = env::predecessor_account_id();
        match self.user_reserves.get(&(user.clone(), tokenIn.clone())) {
            Some(balance) => require!(balance > &maxIn.into()),
            None => panic_str("User does not exist.")
        }

        let amountIn = pool.calculate_out_to_in(&tokenOut, amountOut);
        require!(amountIn <= maxIn);

        self._swap(&user, &tokenIn, amountIn, &tokenOut, amountOut);
    }

    fn _swap(
        &mut self,
        user: &AccountId,
        tokenIn: &AccountId,
        amountIn: U128,
        tokenOut: &AccountId,
        amountOut: U128
    ) {
        let pool = self.get_mut_pool(tokenIn, tokenOut).unwrap();
        pool.incr(&tokenIn, amountIn);
        pool.decr(&tokenOut, amountOut);

        self._withdraw(user,tokenIn, amountIn);

        ext_fungible_token::ext(tokenOut.clone())
            .ft_transfer(
               user.clone(),
                amountOut,
                Some("Transfering B in Swap".to_string())
            );
    }

    fn _deposit(&mut self, user: &AccountId, token: &AccountId, amount: U128) {
        let amount = Balance::from(amount);
        match self.user_reserves.get_mut(&(user.clone(), token.clone())) {
            Some(balance) => *balance += amount,
            None => {
                self.user_reserves.insert((user.clone(), token.clone()), amount);
            }
        };
    }

    fn _withdraw(&mut self, user: &AccountId, token: &AccountId, amount: U128) {
        let amount = Balance::from(amount);
        match self.user_reserves.get_mut(&(user.clone(), token.clone())) {
            Some(balance) => {
                require!(*balance >= Balance::from(amount));
                *balance -= amount
            },
            None => panic_str("No balance to withdraw!")
        };
    }

    fn get_mut_pool(&mut self, tokenIn: &AccountId, tokenOut: &AccountId) -> Option<&mut Pool> {
        match self.pools.get(&(tokenIn.clone(), tokenOut.clone())) {
            Some(p) => self.pools.get_mut(&(tokenIn.clone(), tokenOut.clone())),
            None => self.pools.get_mut(&(tokenOut.clone(), tokenIn.clone()))
        }
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Market {
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token = env::predecessor_account_id();
        let sender_id = AccountId::from(sender_id);
        self._deposit(&sender_id, &token, amount);
        return PromiseOrValue::Value(amount);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::testing_env;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::Balance;
    use near_contract_standards::fungible_token::FungibleToken;

    const OWNER: &str = "owner";
    const NEAR: u128 = 1000000000000000000000000;

    #[test]
    fn initializes() {
        let contract = Market::init(OWNER.parse().unwrap());
        assert_eq!(contract.owner, OWNER.parse().unwrap())
    }

    #[test]
    fn initializes_pool() {
        let mut contract = Market::init(OWNER.parse().unwrap());
        let tokenA: AccountId = "tokena".parse().unwrap();
        let tokenB: AccountId = "tokenb".parse().unwrap();
        contract.init_pool(tokenA.clone(), tokenB.clone());
        assert!(contract.pools.contains_key(&(tokenA, tokenB)))
    }

}
