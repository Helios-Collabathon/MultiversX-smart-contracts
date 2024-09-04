#![no_std]

#[allow(unused_imports)]
use multiversx_sc::imports::*;
use multiversx_sc::derive_imports::*;

const MAX_WALLETS: usize = 25;

type PersonaId = usize;

#[type_abi]
#[derive(NestedDecode, NestedEncode, TopEncode, TopDecode, ManagedVecItem, Clone, Debug, PartialEq, Eq)]
enum Chain {
    MultiversX,
    Injective,
}

#[type_abi]
#[derive(NestedDecode, NestedEncode, TopEncode, TopDecode, Debug, PartialEq, Eq)]
struct Persona<M: ManagedTypeApi> { 
    id: PersonaId,
    wallets: ManagedVec<M, LinkedWallet<M>>,
}

#[type_abi]
#[derive(NestedDecode, NestedEncode, TopEncode, TopDecode, ManagedVecItem, Debug, Clone, PartialEq, Eq)]
struct LinkedWallet<M: ManagedTypeApi> {
    address: ManagedAddress<M>,
    chain: Chain,
}

#[multiversx_sc::contract]
pub trait Identity {
    #[init]
    fn init(&self) {
        self.next_pers_id().set(1);
    }

    #[upgrade]
    fn upgrade(&self) {}

    fn create_persona(&self, caller: &ManagedAddress) {
        let id = self.next_pers_id().get();
        self.next_pers_id().set(id + 1);
        self.owner_lookup(caller.clone()).set(id);
    }

    fn has_persona(&self, caller: &ManagedAddress) -> bool {
        let persona_id = self.owner_lookup(caller.clone()).get();
        persona_id != 0
    }

    fn link_wallet_to_persona(&self, persona_id: PersonaId, chain: &Chain, address: &ManagedAddress) {
        let storage_key = self.get_combined_key(&chain, &address);
        self.persona_lookup(storage_key).set(persona_id);
    }

    fn get_combined_key(&self, chain: &Chain, address: &ManagedAddress) -> ManagedBuffer {
        let mut key = ManagedBuffer::new();
        key.append(address.as_managed_buffer());
        let mut chain_segment = ManagedBuffer::new();
        if chain.top_encode(&mut chain_segment).is_err() {
            sc_panic!("Failed to serialized batch");
        }
        key.append(&chain_segment);
        key
    }

    #[endpoint(addWallet)]
    fn add_wallet(&self, chain: Chain, address: ManagedAddress) {
        let caller = self.blockchain().get_caller();

        require!(caller != address, "Cannot add own address");
    
        if !self.has_persona(&caller) {
            self.create_persona(&caller);
        }
    
        let persona_id = self.owner_lookup(caller).get();
        self.link_wallet_to_persona(persona_id, &chain, &address);
    
        require!(self.persona_wallets(persona_id).len() < MAX_WALLETS, "Max wallets reached");
        require!(!self.persona_wallets(persona_id).contains(&LinkedWallet {
            address: address.clone(),
            chain: chain.clone(),
        }), "Wallet already added");
    
        self.persona_wallets(persona_id).insert(LinkedWallet {
            address,
            chain,
        });
    }

    #[endpoint(removeWallet)]
    fn remove_wallet(&self, chain: Chain, address: ManagedAddress) {
        let caller = self.blockchain().get_caller();
        require!(self.has_persona(&caller), "Persona not found");

        let storage_key = self.get_combined_key(&chain, &address);
        require!(self.persona_lookup(storage_key).get() != 0, "Wallet not found");

        let persona_id = self.owner_lookup(caller).get();

        self.persona_wallets(persona_id).swap_remove(&LinkedWallet {
            address: address.clone(),
            chain: chain.clone(),
        });
        self.persona_lookup(self.get_combined_key(&chain, &address)).clear();
    }

    #[view(getPersonaByAddress)]
    fn get_persona_by_address(&self, chain: Chain, address: ManagedAddress) -> Persona<Self::Api> {
        let storage_key =  self.get_combined_key(&chain, &address);

        let id = self.persona_lookup(storage_key).get();
        let wallets = self.persona_wallets(id).iter().collect::<ManagedVec<LinkedWallet<Self::Api>>>();

        Persona {
            id,
            wallets 
        }
    }

    #[storage_mapper("persona_lookup")]
    fn persona_lookup(&self, storage_key: ManagedBuffer) -> SingleValueMapper<PersonaId>;

    #[storage_mapper("wallets")]
    fn persona_wallets(&self, id: PersonaId) -> UnorderedSetMapper<LinkedWallet<Self::Api>>;

    #[storage_mapper("next_pers_id")]
    fn next_pers_id(&self) -> SingleValueMapper<PersonaId>;

    #[storage_mapper("owner_lookup")]
    fn owner_lookup(&self, owner: ManagedAddress) -> SingleValueMapper<PersonaId>;
}