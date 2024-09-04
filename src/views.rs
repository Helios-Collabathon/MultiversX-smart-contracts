use multiversx_sc::imports::*;

use crate::{chain::Chain, persona::Persona};

#[multiversx_sc::module]
pub trait IdentityViews: crate::storage::IdentityStorage + crate::utils::IdentityUtils {
    #[view(getPersona)]
    fn get_persona(&self, address: ManagedAddress) -> Persona<Self::Api> {
        let persona = self.personas(address).get();

        persona
    }

    #[view(getPersonaByAddress)]
    fn get_persona_by_linked_wallet(&self, chain: Chain, address: ManagedAddress) -> ManagedVec<Self::Api, Persona<Self::Api>> {
        let storage_key =  self.get_combined_key(&chain, &address);
    
        let personas = self.persona_lookup(storage_key)
            .into_iter()
            .map(|address| self.get_persona(address))
            .collect();
    
        personas
    }
}