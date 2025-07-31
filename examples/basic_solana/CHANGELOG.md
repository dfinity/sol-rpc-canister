# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/dfinity/sol-rpc-canister/releases/tag/basic_solana-v1.0.0) - 2025-07-31

### Added

- add client method to estimate recent blockhash ([#121](https://github.com/dfinity/sol-rpc-canister/pull/121))
- method to extract durable nonce from account ([#117](https://github.com/dfinity/sol-rpc-canister/pull/117))
- add client method to sign a transaction ([#113](https://github.com/dfinity/sol-rpc-canister/pull/113))
- add support for `getTokenAccountBalance` RPC method ([#90](https://github.com/dfinity/sol-rpc-canister/pull/90))
- default commitment level for `SolRpcClient` ([#77](https://github.com/dfinity/sol-rpc-canister/pull/77))
- Support method `getBalance` ([#74](https://github.com/dfinity/sol-rpc-canister/pull/74))
- add `sendTransaction` RPC method ([#59](https://github.com/dfinity/sol-rpc-canister/pull/59))
- Implement getSlot RPC method ([#33](https://github.com/dfinity/sol-rpc-canister/pull/33))
- basic Solana wallet example ([#1](https://github.com/dfinity/sol-rpc-canister/pull/1))

### Fixed

- use correct Token Program for SPL in `basic_solana` ([#128](https://github.com/dfinity/sol-rpc-canister/pull/128))

### Other

- improvements to SOL RPC docs ([#158](https://github.com/dfinity/sol-rpc-canister/pull/158))
- add `Custom` Solana network to `basic_solana` canister ([#171](https://github.com/dfinity/sol-rpc-canister/pull/171))
- migrate dependencies to `solana-sdk` repository ([#55](https://github.com/dfinity/sol-rpc-canister/pull/55))
- add build requirements to READMEs and rustdoc  ([#169](https://github.com/dfinity/sol-rpc-canister/pull/169))
- more links ([#168](https://github.com/dfinity/sol-rpc-canister/pull/168))
- improve `basic_solana` Candid documentation ([#161](https://github.com/dfinity/sol-rpc-canister/pull/161))
- use build script for `basic_solana` deployment ([#156](https://github.com/dfinity/sol-rpc-canister/pull/156))
- add support for running `basic_solana` locally and on mainnet ([#91](https://github.com/dfinity/sol-rpc-canister/pull/91))
- update dependencies and bump version ([#145](https://github.com/dfinity/sol-rpc-canister/pull/145))
- add client builder helper methods for `RpcConfig` ([#133](https://github.com/dfinity/sol-rpc-canister/pull/133))
- use 2-out-of-3 threshold consensus for `basic_solana` ([#132](https://github.com/dfinity/sol-rpc-canister/pull/132))
- bump version and use a release notes template ([#130](https://github.com/dfinity/sol-rpc-canister/pull/130))
- update Rust and libraries ([#126](https://github.com/dfinity/sol-rpc-canister/pull/126))
- [**breaking**] use secure primitive types for `Pubkey`, `Signature` and `Hash` ([#98](https://github.com/dfinity/sol-rpc-canister/pull/98))
- remove unused `serde_json` dependency in `basic_solana` ([#94](https://github.com/dfinity/sol-rpc-canister/pull/94))
- clean-up TODOs ([#81](https://github.com/dfinity/sol-rpc-canister/pull/81))
- transfer of SPL token in `basic_solana` ([#78](https://github.com/dfinity/sol-rpc-canister/pull/78))
- integration tests for the `basic_solana` example ([#75](https://github.com/dfinity/sol-rpc-canister/pull/75))
- use SOL RPC canister in `basic_solana` example ([#69](https://github.com/dfinity/sol-rpc-canister/pull/69))
- *(deps)* Bump base64 from 0.13.1 to 0.22.1 ([#11](https://github.com/dfinity/sol-rpc-canister/pull/11))
- *(deps)* use new ic-ed25519 crate ([#7](https://github.com/dfinity/sol-rpc-canister/pull/7))
