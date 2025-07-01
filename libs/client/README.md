[![Internet Computer portal](https://img.shields.io/badge/InternetComputer-grey?logo=internet%20computer&style=for-the-badge)](https://internetcomputer.org)
[![DFinity Forum](https://img.shields.io/badge/help-post%20on%20forum.dfinity.org-blue?style=for-the-badge)](https://forum.dfinity.org/t/sol-rpc-canister/41896)
[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg?logo=apache&style=for-the-badge)](LICENSE)

# Crate `sol_rpc_client`

Library to interact with the [SOL RPC canister](https://github.com/dfinity/sol-rpc-canister/) from a canister running on
the Internet Computer.
See the Rust [documentation](https://docs.rs/sol_rpc_client) for more details.

> ⚠️ **Build Requirements:**
>
> 1. To build this crate, you must copy the `[patch.crates-io]` section from the top-level [`Cargo.toml`](https://github.com/dfinity/sol-rpc-canister/blob/main/Cargo.toml) file in the [`dfinity/sol-rpc`](https://github.com/dfinity/sol-rpc-canister/) repository into your own `Cargo.toml`.  
> This is necessary because the Solana SDK’s `wasm32-unknown-unknown` target assumes a browser environment and depends on `wasm-bindgen`, which is incompatible with use inside a canister.  
> See [this issue](https://github.com/anza-xyz/solana-sdk/issues/117) for details.
>
> 2. On **macOS**, you need an LLVM version that supports the `wasm32-unknown-unknown` target because the Rust [`zstd`](https://docs.rs/zstd/latest/zstd/) crate (used, e.g., to decode `base64+zstd` responses from Solana’s [`getAccountInfo`](https://solana.com/de/docs/rpc/http/getaccountinfo) JSON-RPC method) relies on LLVM during compilation. The default LLVM from Xcode is incompatible. To fix this:
>   * Install LLVM via Homebrew:
>     ```sh
>     brew install llvm
>     ```
>   * Add this to your `.cargo/config.toml`:
>     ```toml
>     [target.'cfg(target_os = "macos")'.env]
>     LLVM_SYS_130_PREFIX = "/opt/homebrew/opt/llvm"
>     ```
>     > 💡 You can find the correct path with:
>     >    ```sh
>     >    brew --prefix llvm
>     >    ```