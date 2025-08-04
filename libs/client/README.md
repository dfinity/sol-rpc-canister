[![Internet Computer portal](https://img.shields.io/badge/InternetComputer-grey?logo=internet%20computer&style=for-the-badge)](https://internetcomputer.org)
[![DFinity Forum](https://img.shields.io/badge/help-post%20on%20forum.dfinity.org-blue?style=for-the-badge)](https://forum.dfinity.org/t/sol-rpc-canister/41896)
[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg?logo=apache&style=for-the-badge)](LICENSE)

# Crate `sol_rpc_client`

Library to interact with the [SOL RPC canister](https://github.com/dfinity/sol-rpc-canister/) from a canister running on
the Internet Computer.
See the Rust [documentation](https://docs.rs/sol_rpc_client) for more details.

## Build Requirements

If you are using the `sol_rpc_types` crate inside a canister, make sure to follow these steps to ensure your code compiles:

1. **Add patch overrides**  
   Copy the `[patch.crates-io]` section from the top-level [`Cargo.toml`](https://github.com/dfinity/sol-rpc-canister/blob/main/Cargo.toml) file in the [`dfinity/sol-rpc`](https://github.com/dfinity/sol-rpc-canister/) repository into your own `Cargo.toml`.  
   This is necessary because the Solana SDK’s `wasm32-unknown-unknown` target assumes a browser environment and depends on `wasm-bindgen`, which is incompatible with canister environments.  
   See [this issue](https://github.com/anza-xyz/solana-sdk/issues/117) for more details.

2. **Override `getrandom` features**  
   Add the following to your `Cargo.toml` file:
   ```toml
   getrandom = { version = "*", features = ["custom"] }
   ```
   This ensures that the `getrandom` crate (a transitive dependency of the Solana SDK) does not pull in `wasm-bindgen`, which is incompatible with canister environments.
   > 💡 You can also specify an exact version of `getrandom`, as long as the `custom` feature is enabled, e.g. `getrandom = { version = "0.2.14", features = ["custom"] }`.

   For more information, see [this blog post](https://forum.dfinity.org/t/module-imports-function-wbindgen-describe-from-wbindgen-placeholder-that-is-not-exported-by-the-runtime/11545/6).
3. **macOS-specific LLVM setup**  
   On **macOS**, an `llvm` version that supports the `wasm32-unknown-unknown` target is required. This is because the `zstd` crate (used, for example, to decode `base64+zstd`-encoded responses from Solana's [`getAccountInfo`](https://solana.com/de/docs/rpc/http/getaccountinfo)) depends on LLVM during compilation. The default LLVM bundled with Xcode does not support `wasm32-unknown-unknown`. To fix this:
   * Install the [Homebrew version](https://formulae.brew.sh/formula/llvm) of LLVM:
     ```sh
     brew install llvm
     ```
   * Create a top-level `.cargo/config.toml` file in your project (or modify the existing one) and add the following entries:
      ```toml
      [env]
      AR="<LLVM_PATH>/bin/llvm-ar"
      CC="<LLVM_PATH>/bin/clang"
      ```
      Where you need to replace `<LLVM_PATH>` with the output of the following command:
      ```sh
      brew --prefix llvm
      ```