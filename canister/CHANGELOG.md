# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-06-12

### Added

- Add support for `transactionDetails=accounts` ([#139](https://github.com/dfinity/sol-rpc-canister/pull/139))
- Add support for `rewards` parameter for `getBlock` ([#135](https://github.com/dfinity/sol-rpc-canister/pull/135))
- Add helper methods for request builders ([#136](https://github.com/dfinity/sol-rpc-canister/pull/136))
- Add client method to sign a transaction ([#113](https://github.com/dfinity/sol-rpc-canister/pull/113))
- Add Chainstack RPC provider ([#118](https://github.com/dfinity/sol-rpc-canister/pull/118))
- Add support for `getSignaturesForAddress` ([#106](https://github.com/dfinity/sol-rpc-canister/pull/106))
- Add support for `getSignatureStatuses` RPC method ([#96](https://github.com/dfinity/sol-rpc-canister/pull/96))
- Add support for `getTokenAccountBalance` RPC method ([#90](https://github.com/dfinity/sol-rpc-canister/pull/90))
- Add support for `getTransaction` RPC method ([#68](https://github.com/dfinity/sol-rpc-canister/pull/68))
- Add `getBlock` RPC method ([#53](https://github.com/dfinity/sol-rpc-canister/pull/53))
- Add `sendTransaction` RPC method ([#59](https://github.com/dfinity/sol-rpc-canister/pull/59))
- Add NOTICE to Apache license ([#60](https://github.com/dfinity/sol-rpc-canister/pull/60))
- Add `getAccountInfo` RPC method ([#49](https://github.com/dfinity/sol-rpc-canister/pull/49))
- Add metrics ([#41](https://github.com/dfinity/sol-rpc-canister/pull/41))
- Add logging crate ([#13](https://github.com/dfinity/sol-rpc-canister/pull/13))
- Add support for override providers ([#12](https://github.com/dfinity/sol-rpc-canister/pull/12))

### Changed

- Bump to 1.0.0
- Inline request parameters `is_default_config` methods ([#138](https://github.com/dfinity/sol-rpc-canister/pull/138))
- Release v0.2.0 ([#131](https://github.com/dfinity/sol-rpc-canister/pull/131))
- Bump version and use a release notes template ([#130](https://github.com/dfinity/sol-rpc-canister/pull/130))
- Candid NonZeroU8 ([#108](https://github.com/dfinity/sol-rpc-canister/pull/108))
- Add `RoundingError` to `sol_rpc_types` ([#105](https://github.com/dfinity/sol-rpc-canister/pull/105))
- Use secure primitive types for `Pubkey`, `Signature` and `Hash` ([#98](https://github.com/dfinity/sol-rpc-canister/pull/98))
- Add support for `getRecentPrioritizationFees` ([#92](https://github.com/dfinity/sol-rpc-canister/pull/92))
- Simplify API keys provisioning script ([#89](https://github.com/dfinity/sol-rpc-canister/pull/89))
- Release v0.1.0 ([#88](https://github.com/dfinity/sol-rpc-canister/pull/88))
- Use `canlog_derive` and `canlog` from crates.io ([#84](https://github.com/dfinity/sol-rpc-canister/pull/84))
- Release pipeline ([#4](https://github.com/dfinity/sol-rpc-canister/pull/4))
- Clean-up TODOs ([#81](https://github.com/dfinity/sol-rpc-canister/pull/81))
- Support method `getBalance` ([#74](https://github.com/dfinity/sol-rpc-canister/pull/74))
- Remove http_types module and use external ic-http-types crate ([#73](https://github.com/dfinity/sol-rpc-canister/pull/73))
- Rename some enum variants to camel case when serializing ([#72](https://github.com/dfinity/sol-rpc-canister/pull/72))
- Use constant size JSON-RPC request ID ([#62](https://github.com/dfinity/sol-rpc-canister/pull/62))
- Use method from JSON-RPC request for metric ([#61](https://github.com/dfinity/sol-rpc-canister/pull/61))
- Cycles cost ([#52](https://github.com/dfinity/sol-rpc-canister/pull/52))
- Client builder ([#54](https://github.com/dfinity/sol-rpc-canister/pull/54))
- Round result from `getSlot` RPC method ([#48](https://github.com/dfinity/sol-rpc-canister/pull/48))
- Use `canhttp` `multi` feature ([#46](https://github.com/dfinity/sol-rpc-canister/pull/46))
- Implement a method for making generic RPC request ([#39](https://github.com/dfinity/sol-rpc-canister/pull/39))
- Implement getSlot RPC method ([#33](https://github.com/dfinity/sol-rpc-canister/pull/33))
- Add some tested RPC providers for Solana Mainnet and Devnet ([#15](https://github.com/dfinity/sol-rpc-canister/pull/15))
- Streamline providers ([#32](https://github.com/dfinity/sol-rpc-canister/pull/32))
- Update rust toolchain to 1.85 ([#21](https://github.com/dfinity/sol-rpc-canister/pull/21))
- Remove unnecessary Storable implementations ([#14](https://github.com/dfinity/sol-rpc-canister/pull/14))
- Add support for API keys ([#10](https://github.com/dfinity/sol-rpc-canister/pull/10))
- Hard-code SOL RPC providers ([#9](https://github.com/dfinity/sol-rpc-canister/pull/9))
- Reproducible build ([#3](https://github.com/dfinity/sol-rpc-canister/pull/3))
- Initial cargo workspace and build pipeline ([#2](https://github.com/dfinity/sol-rpc-canister/pull/2))

### Fixed

- Missing `TraceHttp` logs ([#129](https://github.com/dfinity/sol-rpc-canister/pull/129))
- End-to-end tests for `sendTransaction`  ([#104](https://github.com/dfinity/sol-rpc-canister/pull/104))
- Unit test for `getRecentPrioritizationFees` parameters serialization ([#107](https://github.com/dfinity/sol-rpc-canister/pull/107))
- Integration test for `verifyApiKey` ([#82](https://github.com/dfinity/sol-rpc-canister/pull/82))
- Set `maxSupportedTransactionVersion` to zero for end-to-end tests ([#85](https://github.com/dfinity/sol-rpc-canister/pull/85))
- API keys ([#58](https://github.com/dfinity/sol-rpc-canister/pull/58))
- End-to-end tests ([#45](https://github.com/dfinity/sol-rpc-canister/pull/45))
- Correct Solana cluster for dRPC and Helius providers ([#47](https://github.com/dfinity/sol-rpc-canister/pull/47))
- E2e test with Solana test validator ([#20](https://github.com/dfinity/sol-rpc-canister/pull/20))
- Create test canister on ICP mainnet ([#8](https://github.com/dfinity/sol-rpc-canister/pull/8))

### Removed

- Remove default/non-default providers ([#122](https://github.com/dfinity/sol-rpc-canister/pull/122))


## [0.2.0] - 2025-05-27

### Added

- Add `getRecentPrioritizationFees` RPC method ([#92](https://github.com/dfinity/sol-rpc-canister/pull/92), [#107](https://github.com/dfinity/sol-rpc-canister/pull/107) and [108](https://github.com/dfinity/sol-rpc-canister/pull/108))
- Add `getSignaturesForAddress` RPC method ([#106](https://github.com/dfinity/sol-rpc-canister/pull/106))
- Add `getSignatureStatuses` RPC method ([#96](https://github.com/dfinity/sol-rpc-canister/pull/96))
- Add `getTokenAccountBalance` RPC method ([#90](https://github.com/dfinity/sol-rpc-canister/pull/90))
- Add Chainstack RPC provider ([#118](https://github.com/dfinity/sol-rpc-canister/pull/118))
- End-to-end tests for signing and sending a transaction ([#104](https://github.com/dfinity/sol-rpc-canister/pull/104) and [#114](https://github.com/dfinity/sol-rpc-canister/pull/114))

### Changed

- Move `RoundingError` to `sol_rpc_types` ([#105](https://github.com/dfinity/sol-rpc-canister/pull/105))
- Use secure primitive types for `Pubkey`, `Signature` and `Hash` ([#98](https://github.com/dfinity/sol-rpc-canister/pull/98))

### Fixed

- Missing `TraceHttp` logs ([#129](https://github.com/dfinity/sol-rpc-canister/pull/129))

## [0.1.0] - 2025-04-29

### Added

- Add Solana JSON-RPC providers ([#9](https://github.com/dfinity/sol-rpc-canister/pull/9), [#10](https://github.com/dfinity/sol-rpc-canister/pull/10), [#15](https://github.com/dfinity/sol-rpc-canister/pull/15), [#32](https://github.com/dfinity/sol-rpc-canister/pull/32), [#47](https://github.com/dfinity/sol-rpc-canister/pull/47) and [#58](https://github.com/dfinity/sol-rpc-canister/pull/58))
- Add `getBalance` RPC method ([#74](https://github.com/dfinity/sol-rpc-canister/pull/74))
- Add `getBlock` RPC method ([#53](https://github.com/dfinity/sol-rpc-canister/pull/53))
- Add `getSlot` RPC method ([#33](https://github.com/dfinity/sol-rpc-canister/pull/33) and [#48](https://github.com/dfinity/sol-rpc-canister/pull/48))
- Add `getTransaction` RPC method ([#68](https://github.com/dfinity/sol-rpc-canister/pull/68), [#72](https://github.com/dfinity/sol-rpc-canister/pull/72) and [#81](https://github.com/dfinity/sol-rpc-canister/pull/81))
- Add `sendTransaction` RPC method ([#59](https://github.com/dfinity/sol-rpc-canister/pull/59))
- Add `getAccountInfo` RPC method ([#49](https://github.com/dfinity/sol-rpc-canister/pull/49))
- Add support for making generic JSON-RPC request ([#39](https://github.com/dfinity/sol-rpc-canister/pull/39))
- Add query endpoints for retrieving the cycle costs of RPC methods ([#52](https://github.com/dfinity/sol-rpc-canister/pull/52) and [#62](https://github.com/dfinity/sol-rpc-canister/pull/62))
- Add metrics ([#41](https://github.com/dfinity/sol-rpc-canister/pull/41) and [#61](https://github.com/dfinity/sol-rpc-canister/pull/61))
- Add logging ([#13](https://github.com/dfinity/sol-rpc-canister/pull/13) and [#73](https://github.com/dfinity/sol-rpc-canister/pull/73))
- Add support for override providers for local testing ([#12](https://github.com/dfinity/sol-rpc-canister/pull/12))
- Set `maxSupportedTransactionVersion` to zero for end-to-end tests ([#85](https://github.com/dfinity/sol-rpc-canister/pull/85))
- End-to-end tests ([#20](https://github.com/dfinity/sol-rpc-canister/pull/20) and [#45](https://github.com/dfinity/sol-rpc-canister/pull/45))
