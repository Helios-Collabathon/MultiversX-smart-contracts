use multiversx_sc::imports::*;

use crate::{chain::Chain, persona::Persona};

#[multiversx_sc::module]
pub trait IdentityViews: crate::storage::IdentityStorage + crate::utils::IdentityUtils {
    #[view(getPersona)]
    fn get_persona(&self, address: ManagedAddress) -> OptionalValue<Persona<Self::Api>> {
        if self.personas(address.clone()).is_empty() {
            return OptionalValue::None;
        }

        let persona = self.personas(address).get();

        OptionalValue::Some(persona)
    }

    #[view(getPersonasByAddress)]
    fn get_personas_by_linked_wallet(&self, chain: Chain, address: ManagedBuffer) -> ManagedVec<Self::Api, Persona<Self::Api>> {
        let storage_key =  self.get_combined_key(&chain, &address);
    
        let personas = self.persona_lookup(storage_key)
            .into_iter()
            .map(|address| self.get_persona(address).into_option().unwrap())
            .collect();
    
        personas
    }
}