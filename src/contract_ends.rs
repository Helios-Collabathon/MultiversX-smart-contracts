use multiversx_sc::imports::*;

use crate::{chain::Chain, errors::*, wallet::Wallet};

const MAX_WALLETS: usize = 25;

#[multiversx_sc::module]
pub trait IdentityEndpoints: crate::storage::IdentityStorage + crate::utils::IdentityUtils {
    #[endpoint(addWallet)]
    fn add_wallet(&self, chain: Chain, address: ManagedBuffer) {
        let caller = self.blockchain().get_caller();

        require!(caller.as_managed_buffer().clone() != address, ERROR_CANNOT_ADD_OWN_ADDRESS);
    
        if !self.has_persona(caller.clone()) {
            self.create_persona(caller.clone());
        }
    
        let mut persona = self.personas(caller.clone()).get();
        require!(persona.linked_wallets.len() < MAX_WALLETS, ERROR_MAX_WALLETS_REACHED);
        require!(!persona.linked_wallets.contains(&Wallet {
            address: address.clone(),
            chain: chain.clone(),
        }), ERROR_WALLET_ALREADY_ADDED);
    
        persona.linked_wallets.push(Wallet {
            chain: chain.clone(),
            address: address.clone(),
        });
        self.personas(caller.clone()).set(persona);
        self.link_wallet_to_persona(caller, &chain, &address);
    }

    #[endpoint(removeWallet)]
    fn remove_wallet(&self, chain: Chain, address: ManagedBuffer) {
        let caller = self.blockchain().get_caller();
        require!(self.has_persona(caller.clone()), ERROR_PERSONA_NOT_FOUND);

        let storage_key = self.get_combined_key(&chain, &address);
        require!(self.persona_lookup(storage_key.clone()).contains(&caller), ERROR_WALLET_NOT_FOUND);

        let mut persona = self.personas(caller.clone()).get();
        let index = persona.linked_wallets.iter().position(|wallet| {
            wallet.address == address && wallet.chain == chain
        }).unwrap();
        persona.linked_wallets.remove(index);
        
        self.personas(caller.clone()).set(persona);
        self.persona_lookup(storage_key).clear();
    }
}