//! Simple types to create basic unit tests for the [`crate::SolRpcClient`].
//!
//! Types and methods for this module are only available for non-canister architecture (non `wasm32`).

use crate::ClientBuilder;
use candid::CandidType;
use ic_canister_runtime::StubRuntime;
use sol_rpc_types::{AccountData, AccountEncoding, AccountInfo};

impl<R> ClientBuilder<R> {
    /// Set the runtime to a [`StubRuntime`].
    pub fn with_stub_responses(self) -> ClientBuilder<StubRuntime> {
        self.with_runtime(|_runtime| StubRuntime::default())
    }

    /// Change the runtime to return the given stub response for all calls.
    pub fn with_stub_response<Out: CandidType>(
        self,
        stub_response: Out,
    ) -> ClientBuilder<StubRuntime> {
        self.with_stub_responses().add_stub_response(stub_response)
    }
}

impl ClientBuilder<StubRuntime> {
    /// Change the runtime to return the given stub response for all calls.
    pub fn add_stub_response<Out: CandidType>(
        self,
        stub_response: Out,
    ) -> ClientBuilder<StubRuntime> {
        self.with_runtime(|runtime| runtime.add_stub_response(stub_response))
    }
}

/// USDC token account [`EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`](https://solscan.io/token/EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v) on Solana Mainnet.
pub fn usdc_account() -> AccountInfo {
    AccountInfo {
        lamports: 388_127_047_454,
        data: AccountData::Binary(
            "KLUv/QBYkQIAAQAAAJj+huiNm+Lqi8HMpIeLKYjCQPUrhCS/tA7Rot3LXhmbQLUAvmbxIwAGAQEAAABicKqKWcWUBbRShshncubNEm6bil06OFNtN/e0FOi2Zw==".to_string(),
            AccountEncoding::Base64Zstd,
        ),
        owner: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
        executable: false,
        rent_epoch: 18_446_744_073_709_551_615,
        space: 82,
    }
}

/// Nonce account [`8DedqKHx9ogFajbHtRnTM3pPr3MRyVKDtepEpUiaDXX`](https://explorer.solana.com/address/8DedqKHx9ogFajbHtRnTM3pPr3MRyVKDtepEpUiaDXX?cluster=devnet) on Solana Devnet.
pub fn nonce_account() -> AccountInfo {
    AccountInfo {
        lamports: 1_499_900,
        data: AccountData::Binary("AQAAAAEAAAA+ZK6at2Umwl1p39ifPkNAu66sw5w0AKkY72a19k0LVFBDMPwL0VO7EYlFDc0BAwVcV446FBr/cRWZCGdrPYW9iBMAAAAAAAA=".to_string(), AccountEncoding::Base64),
        owner: "11111111111111111111111111111111".to_string(),
        executable: false,
        rent_epoch: 18_446_744_073_709_551_615,
        space: 80,
    }
}
