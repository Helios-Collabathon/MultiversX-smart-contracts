use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;
use crate::chain::Chain;

#[type_abi]
#[derive(NestedDecode, NestedEncode, TopEncode, TopDecode, ManagedVecItem, Debug, Clone, PartialEq, Eq)]
pub struct Wallet<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub chain: Chain,
}