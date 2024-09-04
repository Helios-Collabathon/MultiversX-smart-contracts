use multiversx_sc::derive_imports::*;
use multiversx_sc::imports::*;
use crate::wallet::Wallet;

#[type_abi]
#[derive(NestedDecode, NestedEncode, TopEncode, TopDecode, Debug, PartialEq, Eq, ManagedVecItem)]
pub struct Persona<M: ManagedTypeApi> { 
    pub linked_wallets: ManagedVec<M, Wallet<M>>,
}