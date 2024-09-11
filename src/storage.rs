use multiversx_sc::imports::*;

use crate::persona::Persona;

#[multiversx_sc::module]
pub trait IdentityStorage: {
    #[storage_mapper("persona_lookup")]
    fn persona_lookup(&self, storage_key: ManagedBuffer) -> UnorderedSetMapper<ManagedAddress>;

    #[storage_mapper("personas")]
    fn personas(&self, owner: ManagedAddress) -> SingleValueMapper<Persona<Self::Api>>;
}