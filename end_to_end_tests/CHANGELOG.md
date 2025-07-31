# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0](https://github.com/dfinity/sol-rpc-canister/releases/tag/sol_rpc_e2e_tests-v1.0.0) - 2025-07-31

### Added

- [**breaking**] add `try_send` method to SOL RPC client ([#187](https://github.com/dfinity/sol-rpc-canister/pull/187))
- add client method to estimate recent blockhash ([#121](https://github.com/dfinity/sol-rpc-canister/pull/121))
- add client method to sign a transaction ([#113](https://github.com/dfinity/sol-rpc-canister/pull/113))

### Other

- migrate dependencies to `solana-sdk` repository ([#55](https://github.com/dfinity/sol-rpc-canister/pull/55))
- add `Cargo.toml` linting to CI pipeline ([#155](https://github.com/dfinity/sol-rpc-canister/pull/155))
- update dependencies and bump version ([#145](https://github.com/dfinity/sol-rpc-canister/pull/145))
- durable nonce end-to-end test ([#124](https://github.com/dfinity/sol-rpc-canister/pull/124))
- add client builder helper methods for `RpcConfig` ([#133](https://github.com/dfinity/sol-rpc-canister/pull/133))
- bump version and use a release notes template ([#130](https://github.com/dfinity/sol-rpc-canister/pull/130))
- use threshold signing in end-to-end tests ([#114](https://github.com/dfinity/sol-rpc-canister/pull/114))
- end-to-end tests for `sendTransaction`  ([#104](https://github.com/dfinity/sol-rpc-canister/pull/104))
