use multiversx_sc::imports::*;

use crate::{chain::Chain, errors::ERROR_TO_CREATE_KEY, persona::Persona};

#[multiversx_sc::module]
pub trait IdentityUtils: crate::storage::IdentityStorage {
    fn create_persona(&self, caller: ManagedAddress) {
        let persona = Persona {
            address: caller.clone(),
            linked_wallets: ManagedVec::new(),
        };
        self.personas(caller).set(persona);
    }

    fn has_persona(&self, caller: ManagedAddress) -> bool {
        !self.personas(caller).is_empty()
    }

    fn link_wallet_to_persona(&self, caller: ManagedAddress, chain: &Chain, address: &ManagedBuffer) {
        let storage_key = self.get_combined_key(chain, address);
        self.persona_lookup(storage_key).insert(caller);
    }

    fn get_combined_key(&self, chain: &Chain, address: &ManagedBuffer) -> ManagedBuffer {
        let mut key = ManagedBuffer::new();
        key.append(address);
        let mut chain_segment = ManagedBuffer::new();
        if chain.top_encode(&mut chain_segment).is_err() {
            sc_panic!(ERROR_TO_CREATE_KEY);
        }
        key.append(&chain_segment);
        key
    }
}