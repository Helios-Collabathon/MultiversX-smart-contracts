#![no_std]

pub mod chain;
pub mod persona;
pub mod wallet;
pub mod views;
pub mod storage;
pub mod utils;
pub mod contract_ends;
pub mod errors;

#[allow(unused_imports)]
use multiversx_sc::imports::*;

#[multiversx_sc::contract]
pub trait Identity: views::IdentityViews + storage::IdentityStorage + utils::IdentityUtils + contract_ends::IdentityEndpoints {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}