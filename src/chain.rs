use core::fmt;

use multiversx_sc::derive_imports::*;

#[type_abi]
#[derive(NestedDecode, NestedEncode, TopEncode, TopDecode, ManagedVecItem, Clone, Debug, PartialEq, Eq)]
pub enum Chain {
    Injective,
    MultiversX
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Chain::Injective => {
                write!(f, "injective")
            }
            Chain::MultiversX => {
                write!(f, "multivers_x")
            }
        }
    }
}