use multiversx_sc::derive_imports::*;

#[type_abi]
#[derive(NestedDecode, NestedEncode, TopEncode, TopDecode, ManagedVecItem, Clone, Debug, PartialEq, Eq)]
pub enum Chain {
    Injective,
    MultiversX
}