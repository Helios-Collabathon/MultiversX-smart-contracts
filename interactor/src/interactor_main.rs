#![allow(non_snake_case)]
#![allow(dead_code)]

mod proxy;

use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};


const GATEWAY: &str = sdk::gateway::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";


#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let _cmd = args.next().expect("at least one argument required");
    let _interact = ContractInteract::new().await;
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    contract_address: Option<Bech32Address>
}

impl State {
        // Deserializes state from file
        pub fn load_state() -> Self {
            if Path::new(STATE_FILE).exists() {
                let mut file = std::fs::File::open(STATE_FILE).unwrap();
                let mut content = String::new();
                file.read_to_string(&mut content).unwrap();
                toml::from_str(&content).unwrap()
            } else {
                Self::default()
            }
        }
    
        /// Sets the contract address
        pub fn set_address(&mut self, address: Bech32Address) {
            self.contract_address = Some(address);
        }
    
        /// Returns the contract address
        pub fn current_address(&self) -> &Bech32Address {
            self.contract_address
                .as_ref()
                .expect("no known contract, deploy first")
        }
    }
    
    impl Drop for State {
        // Serializes state to file
        fn drop(&mut self) {
            let mut file = std::fs::File::create(STATE_FILE).unwrap();
            file.write_all(toml::to_string(self).unwrap().as_bytes())
                .unwrap();
        }
    }

struct ContractInteract {
    interactor: Interactor,
    wallet_address: Address,
    contract_code: BytesValue,
    state: State
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::ivan());
        
        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/identity.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state()
        }
    }

    async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .gas(NumExpr("20,000,000"))
            .typed(proxy::IdentityProxy)
            .init()
            .code(&self.contract_code)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state
            .set_address(Bech32Address::from_bech32_string(new_address_bech32.clone()));

        println!("new address: {new_address_bech32}");
    }

    async fn add_wallet(&mut self, chain: proxy::Chain, address: ManagedAddress<StaticApi>) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(NumExpr("20,000,000"))
            .typed(proxy::IdentityProxy)
            .add_wallet(chain, address)
            .prepare_async()
            .run()
            .await;
    }

    async fn add_wallet_fail(&mut self, chain: proxy::Chain, address: ManagedAddress<StaticApi>, expected_result: ExpectError<'_>) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(NumExpr("20,000,000"))
            .typed(proxy::IdentityProxy)
            .add_wallet(chain, address)
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;
    }

    async fn remove_wallet(&mut self, chain: proxy::Chain, address: ManagedAddress<StaticApi>) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(NumExpr("20,000,000"))
            .typed(proxy::IdentityProxy)
            .remove_wallet(chain, address)
            .prepare_async()
            .run()
            .await;
    }

    async fn remove_wallet_fail(&mut self, chain: proxy::Chain, address: ManagedAddress<StaticApi>, expected_result: ExpectError<'_>) {
        self.interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .gas(NumExpr("20,000,000"))
            .typed(proxy::IdentityProxy)
            .remove_wallet(chain, address)
            .returns(expected_result)
            .prepare_async()
            .run()
            .await;
    }

    async fn get_persona_by_address(&mut self, chain: proxy::Chain, address: ManagedAddress<StaticApi>, result: ManagedVec<StaticApi, proxy::Persona<StaticApi>>) {
        let personas = self
            .interactor
            .query()
            .to(self.state.current_address())
            .typed(proxy::IdentityProxy)
            .get_personas_by_linked_wallet(chain, address)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;
        
        println!("persona: {:?}", personas);
        assert!(ManagedVec::from(personas) == result);
    }
}

#[tokio::test]
async fn test_deploy() {
    let mut interact = ContractInteract::new().await;
    interact.deploy().await;
}

#[tokio::test]
async fn test_get_empty_persona_by_address() {
    let mut interact = ContractInteract::new().await;
    let chain = proxy::Chain::MultiversX;
    let address = ManagedAddress::from(&Address::from_slice(&test_wallets::alice().address().to_bytes()));
    
    interact.get_persona_by_address(chain, address, ManagedVec::new()).await;
}

#[tokio::test]
async fn test_remove__wallet_non_existing_persona() {
    let mut interact = ContractInteract::new().await;
    let chain = proxy::Chain::MultiversX;
    let address = ManagedAddress::from(&Address::from_slice(&test_wallets::alice().address().to_bytes()));

    interact.remove_wallet_fail(chain.clone(), address, ExpectError(4, "Persona not found")).await;
}

#[tokio::test]
async fn test_add_wallet_for_non_existing_persona() {
    let mut interact = ContractInteract::new().await;
    let chain = proxy::Chain::MultiversX;
    let address = ManagedAddress::from(&Address::from_slice(&test_wallets::alice().address().to_bytes()));
    
    let mut persona = proxy::Persona {
        linked_wallets: ManagedVec::new(),
    };
    persona.linked_wallets.push(proxy::Wallet {
        address: address.clone(),
        chain: chain.clone(),
    });
    
    interact.add_wallet(chain.clone(), address.clone()).await;
    interact.get_persona_by_address(chain, address, ManagedVec::from_single_item(persona)).await;
}

#[tokio::test]
async fn test_add_wallet_for_existing_persona() {
    let mut interact = ContractInteract::new().await;
    let chain = proxy::Chain::MultiversX;
    let address = ManagedAddress::from(&Address::from_slice(&test_wallets::bob().address().to_bytes()));
    let old_address = ManagedAddress::from(&Address::from_slice(&test_wallets::alice().address().to_bytes()));
    
    let mut persona = proxy::Persona {
        linked_wallets: ManagedVec::new(),
    };
    persona.linked_wallets.push(proxy::Wallet {
        address: old_address,
        chain: chain.clone(),
    });
    persona.linked_wallets.push(proxy::Wallet {
        address: address.clone(),
        chain: chain.clone(),
    });

    interact.add_wallet(chain.clone(), address.clone()).await;
    interact.get_persona_by_address(chain, address, ManagedVec::from_single_item(persona)).await;
}

#[tokio::test]
async fn test_add_wallet_same_address() {
    let mut interact = ContractInteract::new().await;
    let chain = proxy::Chain::MultiversX;
    let address = ManagedAddress::from(&Address::from_slice(&test_wallets::ivan().address().to_bytes()));

    interact.add_wallet_fail(chain, address, ExpectError(4, "Cannot add own address")).await;
}

#[tokio::test]
async fn test_add_wallet_twice() {
    let mut interact = ContractInteract::new().await;
    let chain = proxy::Chain::MultiversX;
    let address = ManagedAddress::from(&Address::from_slice(&test_wallets::alice().address().to_bytes()));
    
    interact.add_wallet_fail(chain, address, ExpectError(4, "Wallet already added")).await;
}

#[tokio::test]
async fn test_remove_wallet() {
    let mut interact = ContractInteract::new().await;
    let chain = proxy::Chain::MultiversX;
    let address = ManagedAddress::from(&Address::from_slice(&test_wallets::alice().address().to_bytes()));
    let remaining_address = ManagedAddress::from(&Address::from_slice(&test_wallets::bob().address().to_bytes()));

    let mut persona = proxy::Persona {
        linked_wallets: ManagedVec::new(),
    };
    persona.linked_wallets.push(proxy::Wallet {
        address: remaining_address.clone(),
        chain: chain.clone(),
    });

    interact.remove_wallet(chain.clone(), address).await;
    interact.get_persona_by_address(chain, remaining_address, ManagedVec::from_single_item(persona)).await;
}

#[tokio::test]
async fn test_remove_wallet_non_existing_wallet() {
    let mut interact = ContractInteract::new().await;
    let chain = proxy::Chain::MultiversX;
    let address = ManagedAddress::from(&Address::from_slice(&test_wallets::alice().address().to_bytes()));

    interact.remove_wallet_fail(chain.clone(), address, ExpectError(4, "Wallet not found")).await;
}
