# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
