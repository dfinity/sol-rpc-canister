use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{init, update};

#[init]
pub fn init(maybe_init: Option<InitArg>) {
    if let Some(init_arg) = maybe_init {
        todo!()
    }
}

#[update]
pub async fn solana_address(owner: Option<Principal>) -> String {
    "Hello, world!".to_string()
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct InitArg {
    pub solana_network: Option<SolanaNetwork>,
}

#[derive(CandidType, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum SolanaNetwork {
    Mainnet,
    #[default]
    Devnet,
    Testnet
}