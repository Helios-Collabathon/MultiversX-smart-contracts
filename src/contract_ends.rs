use multiversx_sc::imports::*;

use crate::{chain::Chain, wallet::Wallet};

const MAX_WALLETS: usize = 25;

#[multiversx_sc::module]
pub trait IdentityEndpoints: crate::storage::IdentityStorage + crate::utils::IdentityUtils {
    #[endpoint(addWallet)]
    fn add_wallet(&self, chain: Chain, address: ManagedAddress) {
        let caller = self.blockchain().get_caller();

        require!(caller != address, "Cannot add own address");
    
        if !self.has_persona(caller.clone()) {
            self.create_persona(caller.clone());
        }
    
        let mut persona = self.personas(caller.clone()).get();
        require!(persona.linked_wallets.len() < MAX_WALLETS, "Max wallets reached");
        require!(!persona.linked_wallets.contains(&Wallet {
            address: address.clone(),
            chain: chain.clone(),
        }), "Wallet already added");
    
        persona.linked_wallets.push(Wallet {
            address: address.clone(),
            chain: chain.clone(),
        });
        self.personas(caller.clone()).set(persona);
        self.link_wallet_to_persona(caller, &chain, &address);
    }

    #[endpoint(removeWallet)]
    fn remove_wallet(&self, chain: Chain, address: ManagedAddress) {
        let caller = self.blockchain().get_caller();
        require!(self.has_persona(caller.clone()), "Persona not found");

        let storage_key = self.get_combined_key(&chain, &address);
        require!(self.persona_lookup(storage_key).contains(&caller), "Wallet not found");

        let mut persona = self.personas(caller.clone()).get();
        let index = persona.linked_wallets.iter().position(|wallet| {
            wallet.address == address && wallet.chain == chain
        }).unwrap();
        persona.linked_wallets.remove(index);
        
        self.personas(caller.clone()).set(persona);
        self.persona_lookup(self.get_combined_key(&chain, &address)).clear();
    }
}