# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-06-13

### Added

- Add support for `transactionDetails=accounts` ([#139](https://github.com/dfinity/sol-rpc-canister/pull/139))
- Add support for `rewards` parameter for `getBlock` ([#135](https://github.com/dfinity/sol-rpc-canister/pull/135))

## [0.2.0] - 2025-05-27

### Added

- Add `getRecentPrioritizationFees` RPC method ([#92](https://github.com/dfinity/sol-rpc-canister/pull/92), [108](https://github.com/dfinity/sol-rpc-canister/pull/108))
- Add `getSignaturesForAddress` RPC method ([#106](https://github.com/dfinity/sol-rpc-canister/pull/106))
- Add `getSignatureStatuses` RPC method ([#96](https://github.com/dfinity/sol-rpc-canister/pull/96) and [#109](https://github.com/dfinity/sol-rpc-canister/pull/109))
- Add `getTokenAccountBalance` RPC method ([#90](https://github.com/dfinity/sol-rpc-canister/pull/90))
- Add Chainstack RPC provider ([#118](https://github.com/dfinity/sol-rpc-canister/pull/118))
- Add `RoundingError` type ([#105](https://github.com/dfinity/sol-rpc-canister/pull/105))

### Changed

- Use secure primitive types for `Pubkey`, `Signature` and `Hash` ([#98](https://github.com/dfinity/sol-rpc-canister/pull/98))

## [0.1.0] - 2025-04-29

### Added

- Add Solana JSON-RPC providers ([#9](https://github.com/dfinity/sol-rpc-canister/pull/9), [#10](https://github.com/dfinity/sol-rpc-canister/pull/10), [#15](https://github.com/dfinity/sol-rpc-canister/pull/15), [#32](https://github.com/dfinity/sol-rpc-canister/pull/32) and [#47](https://github.com/dfinity/sol-rpc-canister/pull/47))
- Add `getBalance` RPC method ([#74](https://github.com/dfinity/sol-rpc-canister/pull/74))
- Add `getBlock` RPC method ([#53](https://github.com/dfinity/sol-rpc-canister/pull/53))
- Add `getSlot` RPC method ([#33](https://github.com/dfinity/sol-rpc-canister/pull/33) and [#48](https://github.com/dfinity/sol-rpc-canister/pull/48))
- Add `getTransaction` RPC method ([#68](https://github.com/dfinity/sol-rpc-canister/pull/68), [#71](https://github.com/dfinity/sol-rpc-canister/pull/71) and [#72](https://github.com/dfinity/sol-rpc-canister/pull/72))
- Add `sendTransaction` RPC method ([#59](https://github.com/dfinity/sol-rpc-canister/pull/59))
- Add `getAccountInfo` RPC method ([#49](https://github.com/dfinity/sol-rpc-canister/pull/49))
- Add metrics ([#41](https://github.com/dfinity/sol-rpc-canister/pull/41))
- Add logging ([#13](https://github.com/dfinity/sol-rpc-canister/pull/13))
- Add support for override providers for local testing ([#12](https://github.com/dfinity/sol-rpc-canister/pull/12))
