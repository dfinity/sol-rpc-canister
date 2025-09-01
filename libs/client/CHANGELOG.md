# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.0.0] - 2025-08-29

### Changed

- Replace forked `solana-*` crates by the corresponding ones in `solana_sdk` and `agave` version `3.0.0`. The `patch.crates-io` section in `Cargo.toml` is therefore no longer necessary and can be removed. ([#197](https://github.com/dfinity/sol-rpc-canister/pull/197))

## [2.0.0] - 2025-07-31

### Added

- Add `try_send` method to SOL RPC client ([#187](https://github.com/dfinity/sol-rpc-canister/pull/187))

### Changed

- Migrate dependencies to `solana-sdk` repository ([#55](https://github.com/dfinity/sol-rpc-canister/pull/55))
- Enable `ed25519` feature in docs ([#162](https://github.com/dfinity/sol-rpc-canister/pull/162))

### Fixed

- Use correct fee for t-sig with local development key ([#160](https://github.com/dfinity/sol-rpc-canister/pull/160))

## [1.0.0] - 2025-06-13

### Added

- Add a helper method to estimate recent blockhash ([#121](https://github.com/dfinity/sol-rpc-canister/pull/121))
- Add support for `transactionDetails=accounts` ([#139](https://github.com/dfinity/sol-rpc-canister/pull/139))
- Add support for `rewards` parameter for `getBlock` ([#135](https://github.com/dfinity/sol-rpc-canister/pull/135))
- Add client and request builder methods for optional parameters ([#133](https://github.com/dfinity/sol-rpc-canister/pull/133) and [#136](https://github.com/dfinity/sol-rpc-canister/pull/136))

### Fixed

- Fix response estimates for some endpoints ([#147](https://github.com/dfinity/sol-rpc-canister/pull/147))

## [0.2.0] - 2025-05-27

### Added

- Add `getRecentPrioritizationFees` RPC method ([#92](https://github.com/dfinity/sol-rpc-canister/pull/92), [108](https://github.com/dfinity/sol-rpc-canister/pull/108))
- Add `getSignaturesForAddress` RPC method ([#106](https://github.com/dfinity/sol-rpc-canister/pull/106))
- Add `getSignatureStatuses` RPC method ([#96](https://github.com/dfinity/sol-rpc-canister/pull/96))
- Add `getTokenAccountBalance` RPC method ([#90](https://github.com/dfinity/sol-rpc-canister/pull/90))
- Add helper method to sign a transaction ([#113](https://github.com/dfinity/sol-rpc-canister/pull/113))
- Add helper method to extract nonce value from a Solana account ([#117](https://github.com/dfinity/sol-rpc-canister/pull/117))

### Changed

- Use secure primitive types for `Pubkey`, `Signature` and `Hash` ([#98](https://github.com/dfinity/sol-rpc-canister/pull/98))
- Use `RoundingError` from `sol_rpc_types` ([#105](https://github.com/dfinity/sol-rpc-canister/pull/105))

## [0.1.0] - 2025-04-29

### Added

- Add Solana JSON-RPC providers ([#9](https://github.com/dfinity/sol-rpc-canister/pull/9), [#10](https://github.com/dfinity/sol-rpc-canister/pull/10) and [#32](https://github.com/dfinity/sol-rpc-canister/pull/32))
- Add `getBalance` RPC method ([#74](https://github.com/dfinity/sol-rpc-canister/pull/74))
- Add `getBlock` RPC method ([#53](https://github.com/dfinity/sol-rpc-canister/pull/53))
- Add `getSlot` RPC method ([#33](https://github.com/dfinity/sol-rpc-canister/pull/33) and [#48](https://github.com/dfinity/sol-rpc-canister/pull/48))
- Add `getTransaction` RPC method ([#68](https://github.com/dfinity/sol-rpc-canister/pull/68))
- Add `sendTransaction` RPC method ([#59](https://github.com/dfinity/sol-rpc-canister/pull/59))
- Add `getAccountInfo` RPC method ([#49](https://github.com/dfinity/sol-rpc-canister/pull/49))
- Add support for making generic JSON-RPC request ([#39](https://github.com/dfinity/sol-rpc-canister/pull/39))
- Retrieve the cycle costs of RPC methods ([#52](https://github.com/dfinity/sol-rpc-canister/pull/52))
- Use the builder pattern for instantiating the client ([#54](https://github.com/dfinity/sol-rpc-canister/pull/54))
- Use a default commitment level for requests made by the client ([#77](https://github.com/dfinity/sol-rpc-canister/pull/77))
- Add Rust documentation tests for the client ([#65](https://github.com/dfinity/sol-rpc-canister/pull/65))

[3.0.0]: https://github.com/dfinity/sol-rpc-canister/compare/sol_rpc_client-v2.0.0...sol_rpc_client-v3.0.0
[2.0.0]: https://github.com/dfinity/sol-rpc-canister/compare/v1.0.0...sol_rpc_client-v2.0.0
[1.0.0]: https://github.com/dfinity/sol-rpc-canister/compare/v0.2.0...v1.0.0
[0.2.0]: https://github.com/dfinity/sol-rpc-canister/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/dfinity/sol-rpc-canister/releases/tag/v0.1.0
