#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen};
const DEFAULT_MESSAGE: &str = "Hello";
pub struct Contract {
    message: String,
}
impl borsh::de::BorshDeserialize for Contract
where
    String: borsh::BorshDeserialize,
{
    fn deserialize(buf: &mut &[u8]) -> ::core::result::Result<Self, borsh::maybestd::io::Error> {
        Ok(Self {
            message: borsh::BorshDeserialize::deserialize(buf)?,
        })
    }
}
impl borsh::ser::BorshSerialize for Contract
where
    String: borsh::ser::BorshSerialize,
{
    fn serialize<W: borsh::maybestd::io::Write>(
        &self,
        writer: &mut W,
    ) -> ::core::result::Result<(), borsh::maybestd::io::Error> {
        borsh::BorshSerialize::serialize(&self.message, writer)?;
        Ok(())
    }
}
#[must_use]
pub struct ContractExt {
    pub(crate) account_id: near_sdk::AccountId,
    pub(crate) deposit: near_sdk::Balance,
    pub(crate) static_gas: near_sdk::Gas,
    pub(crate) gas_weight: near_sdk::GasWeight,
}
impl ContractExt {
    pub fn with_attached_deposit(mut self, amount: near_sdk::Balance) -> Self {
        self.deposit = amount;
        self
    }
    pub fn with_static_gas(mut self, static_gas: near_sdk::Gas) -> Self {
        self.static_gas = static_gas;
        self
    }
    pub fn with_unused_gas_weight(mut self, gas_weight: u64) -> Self {
        self.gas_weight = near_sdk::GasWeight(gas_weight);
        self
    }
}
impl Contract {
    /// API for calling this contract's functions in a subsequent execution.
    pub fn ext(account_id: near_sdk::AccountId) -> ContractExt {
        ContractExt {
            account_id,
            deposit: 0,
            static_gas: near_sdk::Gas(0),
            gas_weight: near_sdk::GasWeight::default(),
        }
    }
}
impl Default for Contract {
    fn default() -> Self {
        Self {
            message: DEFAULT_MESSAGE.to_string(),
        }
    }
}
impl ContractExt {
    pub fn get_greeting(self) -> near_sdk::Promise {
        let __args = ::alloc::vec::Vec::new();
        near_sdk::Promise::new(self.account_id).function_call_weight(
            "get_greeting".to_string(),
            __args,
            self.deposit,
            self.static_gas,
            self.gas_weight,
        )
    }
    pub fn set_greeting(self, message: String) -> near_sdk::Promise {
        let __args = {
            #[serde(crate = "near_sdk::serde")]
            struct Input<'nearinput> {
                message: &'nearinput String,
            }
            #[doc(hidden)]
            #[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
            const _: () = {
                use near_sdk::serde as _serde;
                #[automatically_derived]
                impl<'nearinput> near_sdk::serde::Serialize for Input<'nearinput> {
                    fn serialize<__S>(
                        &self,
                        __serializer: __S,
                    ) -> near_sdk::serde::__private::Result<__S::Ok, __S::Error>
                    where
                        __S: near_sdk::serde::Serializer,
                    {
                        let mut __serde_state = match _serde::Serializer::serialize_struct(
                            __serializer,
                            "Input",
                            false as usize + 1,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        match _serde::ser::SerializeStruct::serialize_field(
                            &mut __serde_state,
                            "message",
                            &self.message,
                        ) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        };
                        _serde::ser::SerializeStruct::end(__serde_state)
                    }
                }
            };
            let __args = Input { message: &message };
            near_sdk::serde_json::to_vec(&__args)
                .expect("Failed to serialize the cross contract args using JSON.")
        };
        near_sdk::Promise::new(self.account_id).function_call_weight(
            "set_greeting".to_string(),
            __args,
            self.deposit,
            self.static_gas,
            self.gas_weight,
        )
    }
}
impl Contract {
    pub fn get_greeting(&self) -> String {
        return self.message.clone();
    }
    pub fn set_greeting(&mut self, message: String) {
        ::near_sdk::env::log_str(
            {
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["Saving greeting "],
                    &[::core::fmt::ArgumentV1::new_display(&message)],
                ));
                res
            }
            .as_str(),
        );
        self.message = message;
    }
}
