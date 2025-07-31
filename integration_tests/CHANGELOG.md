# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/dfinity/sol-rpc-canister/releases/tag/sol_rpc_int_tests-v1.0.0) - 2025-07-31

### Added

- [**breaking**] add `try_send` method to SOL RPC client ([#187](https://github.com/dfinity/sol-rpc-canister/pull/187))
- Select supported providers based on successful responses ([#183](https://github.com/dfinity/sol-rpc-canister/pull/183))
- add client method to estimate recent blockhash ([#121](https://github.com/dfinity/sol-rpc-canister/pull/121))
- add support for `transactionDetails=accounts` ([#139](https://github.com/dfinity/sol-rpc-canister/pull/139))
- add support for `rewards` parameter for `getBlock` ([#135](https://github.com/dfinity/sol-rpc-canister/pull/135))
- add client method to sign a transaction ([#113](https://github.com/dfinity/sol-rpc-canister/pull/113))
- Candid NonZeroU8 ([#108](https://github.com/dfinity/sol-rpc-canister/pull/108))
- add support for `getSignaturesForAddress` ([#106](https://github.com/dfinity/sol-rpc-canister/pull/106))
- add support for `getSignatureStatuses` RPC method ([#96](https://github.com/dfinity/sol-rpc-canister/pull/96))
- Add support for `getRecentPrioritizationFees` ([#92](https://github.com/dfinity/sol-rpc-canister/pull/92))
- add support for `getTokenAccountBalance` RPC method ([#90](https://github.com/dfinity/sol-rpc-canister/pull/90))
- Support method `getBalance` ([#74](https://github.com/dfinity/sol-rpc-canister/pull/74))
- add support for `getTransaction` RPC method ([#68](https://github.com/dfinity/sol-rpc-canister/pull/68))
- add `getBlock` RPC method ([#53](https://github.com/dfinity/sol-rpc-canister/pull/53))
- add `sendTransaction` RPC method ([#59](https://github.com/dfinity/sol-rpc-canister/pull/59))
- cycles cost ([#52](https://github.com/dfinity/sol-rpc-canister/pull/52))
- add `getAccountInfo` RPC method ([#49](https://github.com/dfinity/sol-rpc-canister/pull/49))
- client builder ([#54](https://github.com/dfinity/sol-rpc-canister/pull/54))
- round result from `getSlot` RPC method ([#48](https://github.com/dfinity/sol-rpc-canister/pull/48))
- use `canhttp` `multi` feature ([#46](https://github.com/dfinity/sol-rpc-canister/pull/46))
- Implement a method for making generic RPC request ([#39](https://github.com/dfinity/sol-rpc-canister/pull/39))
- Implement getSlot RPC method ([#33](https://github.com/dfinity/sol-rpc-canister/pull/33))
- add logging crate ([#13](https://github.com/dfinity/sol-rpc-canister/pull/13))
- add support for override providers ([#12](https://github.com/dfinity/sol-rpc-canister/pull/12))
- Add support for API keys ([#10](https://github.com/dfinity/sol-rpc-canister/pull/10))
- hard-code SOL RPC providers ([#9](https://github.com/dfinity/sol-rpc-canister/pull/9))

### Fixed

- missing `TraceHttp` logs ([#129](https://github.com/dfinity/sol-rpc-canister/pull/129))

### Other

- do not record metrics for requests with insufficient cycles ([#184](https://github.com/dfinity/sol-rpc-canister/pull/184))
- require HTTP outcall base fee ([#185](https://github.com/dfinity/sol-rpc-canister/pull/185))
- add more metrics ([#144](https://github.com/dfinity/sol-rpc-canister/pull/144))
- migrate dependencies to `solana-sdk` repository ([#55](https://github.com/dfinity/sol-rpc-canister/pull/55))
- integration test for fetching metrics ([#143](https://github.com/dfinity/sol-rpc-canister/pull/143))
- update dependencies and bump version ([#145](https://github.com/dfinity/sol-rpc-canister/pull/145))
- Revisit response size estimates ([#147](https://github.com/dfinity/sol-rpc-canister/pull/147))
- add missing documentation for  `getTransaction` ([#137](https://github.com/dfinity/sol-rpc-canister/pull/137))
- add helper methods for request builders ([#136](https://github.com/dfinity/sol-rpc-canister/pull/136))
- add client builder helper methods for `RpcConfig` ([#133](https://github.com/dfinity/sol-rpc-canister/pull/133))
- bump version and use a release notes template ([#130](https://github.com/dfinity/sol-rpc-canister/pull/130))
- update Rust and libraries ([#126](https://github.com/dfinity/sol-rpc-canister/pull/126))
- add Chainstack RPC provider ([#118](https://github.com/dfinity/sol-rpc-canister/pull/118))
- end-to-end tests for `sendTransaction`  ([#104](https://github.com/dfinity/sol-rpc-canister/pull/104))
- [**breaking**] use secure primitive types for `Pubkey`, `Signature` and `Hash` ([#98](https://github.com/dfinity/sol-rpc-canister/pull/98))
- use default commitment for client in `solana_test_validator.rs` ([#97](https://github.com/dfinity/sol-rpc-canister/pull/97))
- add integration test for `getTokenAccountBalance` ([#95](https://github.com/dfinity/sol-rpc-canister/pull/95))
- integration test for `verifyApiKey` ([#82](https://github.com/dfinity/sol-rpc-canister/pull/82))
- use `canlog_derive` and `canlog` from crates.io ([#84](https://github.com/dfinity/sol-rpc-canister/pull/84))
- *(http-types)* Remove http_types module and use external ic-http-types crate ([#73](https://github.com/dfinity/sol-rpc-canister/pull/73))
- use SOL RPC canister to fetch blockhash in integration test ([#67](https://github.com/dfinity/sol-rpc-canister/pull/67))
- use constant size JSON-RPC request ID ([#62](https://github.com/dfinity/sol-rpc-canister/pull/62))
- add NOTICE to Apache license ([#60](https://github.com/dfinity/sol-rpc-canister/pull/60))
- Forward calls through wallet canister ([#40](https://github.com/dfinity/sol-rpc-canister/pull/40))
- Add some tested RPC providers for Solana Mainnet and Devnet ([#15](https://github.com/dfinity/sol-rpc-canister/pull/15))
- Streamline providers ([#32](https://github.com/dfinity/sol-rpc-canister/pull/32))
- e2e test with Solana test validator ([#20](https://github.com/dfinity/sol-rpc-canister/pull/20))
- *(deps)* update Pocket IC ([#31](https://github.com/dfinity/sol-rpc-canister/pull/31))
- update rust toolchain to 1.85 ([#21](https://github.com/dfinity/sol-rpc-canister/pull/21))
- initial cargo workspace and build pipeline ([#2](https://github.com/dfinity/sol-rpc-canister/pull/2))
