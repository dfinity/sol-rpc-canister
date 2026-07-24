# Basic Solana

## Overview

This example demonstrates how to deploy an application on the Internet Computer
(known as [canisters](https://docs.internetcomputer.org/concepts/canisters/)) **that can control digital
assets** on the Solana blockchain:
1. SOL, the native currency on Solana;
2. any other token (known as [SPL tokens](https://solana.com/docs/tokens)).

:movie_camera: Check out also this [demo](https://youtu.be/CpxQqp6CxoY?feature=shared) that runs through most parts of this example.

## Architecture

This example internally leverages:
- [Threshold Schnorr](https://docs.internetcomputer.org/concepts/chain-key-cryptography/#chain-key-signatures-threshold-ecdsa-and-schnorr): Each user's Solana address is derived deterministically from the canister's threshold Ed25519 key using a derivation path based on the user's IC principal. This means each user has a unique, stable Solana address controlled by the canister.
- [HTTPS outcalls](https://docs.internetcomputer.org/concepts/https-outcalls): The canister communicates with the Solana network via the [SOL RPC canister](https://github.com/dfinity/sol-rpc-canister) (canister ID `tghme-zyaaa-aaaar-qarca-cai` on ICP mainnet), which forwards requests to Solana RPC providers.

For a deeper understanding of the ICP ↔ SOL integration, see the [chain fusion overview](https://docs.internetcomputer.org/concepts/chain-fusion/solana/).

## Deploying from ICP Ninja

The quickest way to try this example — no local setup required — is to deploy it directly from your browser with [ICP Ninja](https://icp.ninja):

[![](https://icp.ninja/assets/open.svg)](https://icp.ninja/editor?g=https://github.com/dfinity/sol-rpc-canister/tree/main/examples/basic_solana)

When deployed from ICP Ninja, only the `backend` canister is deployed; it connects to **Solana Devnet** through the [SOL RPC canister](https://github.com/dfinity/sol-rpc-canister) already running on ICP mainnet (`tghme-zyaaa-aaaar-qarca-cai`), rather than deploying a local one. Note that canisters deployed via ICP Ninja remain live for 50 minutes after signing in with your Internet Identity.

To interact with the endpoints, you have two options:
- Open the canister's **Candid UI** from the ICP Ninja interface and sign in with **Internet Identity**. The account-deriving methods reject the anonymous principal, so you must be signed in.
- Use icp-cli to call the deployed canister by its principal with the `-n ic` flag (and a non-anonymous identity, see [Interacting with the deployed canister](#interacting-with-the-deployed-canister)):

  ```bash
  icp canister call -n ic <BACKEND_CANISTER_ID> solana_account '(null)'
  ```

  where `<BACKEND_CANISTER_ID>` is the principal shown for the `backend` canister in the ICP Ninja interface.

## Build and deploy from the command line

### Prerequisites

- A [Rust toolchain](https://www.rust-lang.org/tools/install) (installed via `rustup`). The `wasm32-unknown-unknown` target is added automatically from [`rust-toolchain.toml`](rust-toolchain.toml).
- [icp-cli](https://cli.internetcomputer.org). It can be installed in several ways — see the [installation guide](https://cli.internetcomputer.org) for all options. For example, via npm (requires Node.js):

  ```bash
  npm install -g @icp-sdk/icp-cli @icp-sdk/ic-wasm
  ```

> [!IMPORTANT]
> On **macOS**, an `llvm` version that supports the `wasm32-unknown-unknown` target is required, because the `zstd` crate (used to decode Solana's `base64+zstd` responses) compiles C code for wasm and Apple's bundled `clang` cannot target it. Install the [Homebrew version](https://formulae.brew.sh/formula/llvm) with `brew install llvm`, and point the C compiler at it before building/deploying:
> ```bash
> export AR="$(brew --prefix llvm)/bin/llvm-ar"
> export CC="$(brew --prefix llvm)/bin/clang"
> ```

> [!NOTE]
> If you wish to use this example as a starting point for your own project, make sure you follow the instructions in the [build requirements](https://github.com/dfinity/sol-rpc-canister/blob/main/libs/client/README.md#build-requirements) for the `sol_rpc_client` crate to ensure that your code compiles.

### Install

```bash
git clone https://github.com/dfinity/sol-rpc-canister
cd sol-rpc-canister/examples/basic_solana
```

### Deploy and test locally

The local icp-cli network supports real HTTPS outcalls, so all operations work against live Solana Devnet data
without deploying to ICP mainnet.

```bash
icp network start -d
icp deploy
```

Keep the network running while you work through the [interaction steps](#interacting-with-the-deployed-canister)
below. When you are done, stop it with `icp network stop`.

`icp deploy` deploys two canisters locally:
- `backend`: the Solana wallet canister described in this example;
- `sol_rpc`: the [SOL RPC canister](https://github.com/dfinity/sol-rpc-canister), which the backend uses to talk to Solana.

The backend reads the SOL RPC canister's principal from the `PUBLIC_CANISTER_ID:sol_rpc` canister environment
variable, which icp-cli injects automatically after deploying the `sol_rpc` canister. Locally, the backend talks to
Solana Devnet through a custom RPC endpoint (`https://api.devnet.solana.com`) — see [`icp.yaml`](icp.yaml) for the
environment configuration.

### Deploy to ICP mainnet

```bash
icp deploy -e ic
```

This deploys only the `backend` canister and points it to the [shared SOL RPC canister](https://github.com/dfinity/sol-rpc-canister)
(`tghme-zyaaa-aaaar-qarca-cai`) already running on ICP mainnet, which is injected via the `PUBLIC_CANISTER_ID:sol_rpc`
environment variable. Deploying to ICP mainnet requires
[cycles](https://docs.internetcomputer.org/concepts/cycles).

The default configuration interacts with **Solana Devnet** using the Internet Computer's `test_key_1` Ed25519 test key
— suitable for testing with free Devnet SOL from a faucet. To deploy for production use on Solana Mainnet, update the
`init_args` for the `backend` canister in the `ic` environment in [`icp.yaml`](icp.yaml) to use `variant { Mainnet }`
and the production key `"key_1"` — see the comment in `icp.yaml` for the exact value. To learn more about the
initialization arguments, see the `InitArg` type in [`backend/lib.rs`](backend/lib.rs).

> [!IMPORTANT]
> The commands in the following sections target the local deployment by default. To target your ICP mainnet
> deployment instead, add the `-e ic` flag to every `icp canister call` command.

## Interacting with the deployed canister

> [!IMPORTANT]
> icp-cli uses the **anonymous** identity by default. The methods below derive a Solana account
> from the caller's principal and **reject the anonymous principal**, so you first need to create
> and select a non-anonymous identity:
>
> ```bash
> icp identity new my-wallet
> icp identity default my-wallet
> ```
>
> All commands in this section then run as `my-wallet`. You can switch identities at any time with
> `icp identity default <name>`, or select one per call with the `--identity <name>` flag.

### Step 1: Generating a Solana account

A Solana account can be derived from an EdDSA public key. To derive a user's specific account, identified on the IC by a
principal, the canister uses its own threshold EdDSA public key to derive a new public key deterministically for each
requested principal. To retrieve your Solana account, you can call the `solana_account` method on the previously
deployed canister:

```bash
icp canister call backend solana_account '(null)'
```

This will return a Solana account such as `("2kqg1tEj59FNe3hSiLH88SySB9D7fUSArum6TP6iHFQY")` that is tied to your
principal. Your account will be different. You can view such accounts on any Solana explorer such
as [Solana Explorer](https://explorer.solana.com/?cluster=devnet).

If you want to send some SOL to someone else, you can also use the above method to enquire about their Solana account
given their IC principal:

```bash
icp canister call backend solana_account '(opt principal "hkroy-sm7vs-yyjs7-ekppe-qqnwx-hm4zf-n7ybs-titsi-k6e3k-ucuiu-uqe")'
```

This will return a different Solana address as the one above, such as
`("8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq")`.

### Step 2: Receiving SOL

Now that you have your Solana account, let us send some Devnet SOL to it:

1. Get some Devnet SOL if you don't have any. You can for example use [this faucet](https://faucet.solana.com/).
2. Send some Devnet SOL to the address you obtained in the previous step. You can use any Solana wallet to do so.

Once the transaction is confirmed, you'll be able to see it in your Solana account's balance, which should be visible in
a Solana explorer, e.g. https://explorer.solana.com/address/2kqg1tEj59FNe3hSiLH88SySB9D7fUSArum6TP6iHFQY?cluster=devnet.

You can also query the balance (in Lamports) directly from the canister:

```bash
icp canister call backend get_balance '(null)'
```

### Step 3: Sending SOL

You can send SOL using the `send_sol` endpoint on your canister, specifying a Solana destination account and an amount
in the smallest unit (Lamport). For example, to send 1_000_000 Lamports (0.001 SOL) to
`8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq`, run the following command:

> [!NOTE]
> If no principal is provided, the caller's principal is used. In this example, you could replace `null` with another principal to send SOL on their behalf. This is behaviour you would typically not want in production, as it allows anyone to send SOL from any account to any other account. In production, you would typically want to restrict the `send_sol` endpoint to only allow sending SOL from the caller's account.

```bash
# send_sol returns the transaction ID (the transaction's first signature) as a Candid string,
# e.g. ("3jAZAYQbG4646gogXewsK28ZgGGBPBPH3fAKocYXr3DXwz7shQRx2rVmBTQZnSpuT9..").
# Extract the base58 signature and print a link to Solana Explorer:
txid=$(icp canister call backend send_sol '(null, "8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq", 1_000_000)' | grep -oE '[1-9A-HJ-NP-Za-km-z]{64,}')
echo "https://explorer.solana.com/tx/$txid?cluster=devnet"
```

> [!NOTE]
> Solana requires every account to hold a minimum balance to be [rent-exempt](https://solana.com/docs/core/accounts#rent) — about 0.00089 SOL (890880 Lamports) for a basic account. The **first** transfer to a new, empty account must therefore be at least this amount, otherwise the transaction fails with a `Transaction results in an account with insufficient funds for rent` error.

The `send_sol` endpoint sends SOL by executing the following steps:

1. Retrieving a [recent blockhash](https://solana.com/docs/core/transactions#recent-blockhash). This is necessary
   because all Solana transactions must include a blockhash within the 151 most recent stored hashes (which corresponds
   to about 60 to 90 seconds).
2. Building a Solana [transaction](https://solana.com/docs/core/transactions) that includes a single instruction to
   transfer the specified amount from the sender's address to the given receiver's address, as well as the recent
   blockhash.
3. Signing the Solana transaction using
   the [threshold Ed25519 API](https://docs.internetcomputer.org/references/management-canister/#chain-key-signing).
4. Sending the signed transaction to the Solana network using the `sendTransaction` method in
   the [SOL RPC canister](https://github.com/dfinity/sol-rpc-canister).

Opening the Solana Explorer link printed above lets you follow the transfer's status on the Solana network.

Once the transaction is confirmed, you can verify the recipient's new balance by passing its address to `get_balance`
(pass `opt "<ADDRESS>"` to query any account, or `null` to query your own):

```bash
icp canister call backend get_balance '(opt "8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq")'
```

### Step 4: Sending SOL using durable nonces

[Durable nonces](https://solana.com/developers/guides/advanced/introduction-to-durable-nonces) can be used instead of a
recent blockhash when constructing a Solana transaction. This can be useful for example when signing a transaction in
advance before sending it out.

In order to use durable nonces, you must first create a nonce account controlled by your Solana account. The nonce
account contains the current value of the durable nonce. To create a nonce account controlled by your Solana account,
run the following command:

```bash
icp canister call backend create_nonce_account '(null)'
```

You can inspect the created nonce account and get the current durable nonce value in a Solana explorer. You can also
fetch the current value of the durable nonce by running the following command:

```bash
icp canister call backend get_nonce '(null)'
```

To send some SOL using a durable nonce, you can run the following command:

> [!NOTE]
> If no principal is provided, the caller's principal is used. In this example, you could replace `null` with another principal to send SOL on their behalf. This is behaviour you would typically not want in production, as it allows anyone to send SOL from any account to any other account. In production, you would typically want to restrict the `send_sol_with_durable_nonce` endpoint to only allow sending SOL from the caller's account.

```bash
icp canister call backend send_sol_with_durable_nonce '(null, "8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq", 1)'
```

The `send_sol_with_durable_nonce` endpoint works similarly to the `send_sol` endpoint, however the instructions included
in the transaction are different and the durable nonce is included in the transaction instead of a recent blockhash. The
`send_sol_with_durable_nonce` endpoint sends SOL by executing the following steps:

1. Retrieving the current durable nonce value from the nonce account.
2. Building a Solana [transaction](https://solana.com/docs/core/transactions) that includes instructions to
    1. [advance the nonce account](https://solana.com/developers/guides/advanced/introduction-to-durable-nonces#advancing-nonce)
       (which is required so that the nonce value is used only once), and
    2. transfer the specified amount from the sender's address to the given receiver's address,

   as well as the durable nonce value instead of a recent blockhash.
3. Signing the Solana transaction using
   the [threshold Ed25519 API](https://docs.internetcomputer.org/references/management-canister/#chain-key-signing).
4. Sending the signed transaction to the Solana network using the `sendTransaction` method in
   the [SOL RPC canister](https://github.com/dfinity/sol-rpc-canister).

The `send_sol_with_durable_nonce` endpoint returns the transaction ID of the transaction sent to the Solana network. You
can also verify (either in a Solana explorer or using the `get_nonce` endpoint) that the nonce value stored in the
account has changed after calling this endpoint.

### Step 5: Sending Solana Program Library (SPL) tokens

We will now be sending some SPL tokens on Solana Devnet. The instructions below work for any SPL token. You may for
example use the USDC token whose mint account on Devnet is `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`.

You first need to create [Associated Token Accounts (ATA)](https://spl.solana.com/associated-token-account) for the
sender and recipient accounts if they do not exist yet. An ATA is
a [Program Derived Address (PDA)](https://solana.com/docs/core/pda) derived from a Solana account using the token mint
account. An ATA is needed for each type of SPL token held by a Solana account.

We create two new identities, one for the sender and one for the recipient. You can do this by running the following commands:

```bash
icp identity new sender
icp identity new recipient
```

We have to make sure the Solana accounts belonging to the new identities created above actually hold SOL to pay for transaction fees. For this, follow the instructions outlined in [Step 1](#step-1-generating-a-solana-account) and [Step 2](#step-2-receiving-sol) for each identity. You can switch between identities using the `icp identity default <IDENTITY_NAME>` command or specify the identity to use by adding the `--identity <IDENTITY_NAME>` flag to the `icp canister call` commands.

To create the ATAs for the sender and
recipient, you can run the following commands:

> [!NOTE]
> If no principal is provided as the first argument, the caller's principal is used.

```bash
icp canister call --identity sender backend create_associated_token_account '(null, "<TOKEN MINT ADDRESS>")'
icp canister call --identity recipient backend create_associated_token_account '(null, "<TOKEN MINT ADDRESS>")'
```

This works by sending transactions that instruct the
Solana [Associated Token Account Program](https://spl.solana.com/associated-token-account) to create and initialize
these accounts. You can now inspect the sender and recipient accounts on a Solana explorer and confirm that you can see
a balance of 0 for the corresponding SPL token.

To send some tokens from the sender to the receiver, you will need to obtain some tokens on the sender account (using
e.g. [this faucet](https://faucet.circle.com/) for USDC). To do this, you will need the ATA address of the sender. You
can for example get it by running the following command:

> [!NOTE]
> If no principal is provided as the first argument, the caller's principal is used.

```bash
icp canister call --identity sender backend associated_token_account '(null, "<TOKEN MINT ADDRESS>")'
```

To transfer some tokens from the sender to the recipient, you can run the following command:

> [!NOTE]
> If no principal is provided as the first argument, the caller's principal is used.
> Make sure to use the `RECIPIENT SOLANA ADDRESS`, not their ATA.

```bash
icp canister call --identity sender backend send_spl_token '(null, "<TOKEN MINT ADDRESS>", "<RECIPIENT SOLANA ADDRESS>", <AMOUNT>)'
```

The `send_spl_token` endpoint works similarly to the `send_sol` endpoint, but creates a transaction with the sender and
recipient ATAs instead of their account addresses. You can also inspect the resulting transaction on a Solana explorer,
and verify that the associated token balances were updated accordingly. You can also check the updated token balances by
running the following commands:

> [!NOTE]
> If no ATA is provided, it is derived from the caller's principal.

```bash
icp canister call backend get_spl_token_balance '(opt "<SENDER ATA>", "<TOKEN MINT ADDRESS>")'
icp canister call backend get_spl_token_balance '(opt "<RECIPIENT ATA>", "<TOKEN MINT ADDRESS>")'
```

## Testing

The example ships with an integration test (`backend/tests/tests.rs`) that exercises all the endpoints end to end. It
spins up the `backend` and SOL RPC canisters with [PocketIC](https://github.com/dfinity/pocketic) and runs them
against a local Solana test validator, to which all RPC traffic is redirected.

Prerequisites:
- The [Solana CLI](https://solana.com/docs/intro/installation/dependencies#install-solana-cli), which provides the `solana-test-validator` binary.
- The PocketIC server is **downloaded automatically** on first run (so the first `cargo test` may take a while). To use a local copy instead, set the [`POCKET_IC_BIN`](https://github.com/dfinity/pocketic) environment variable.

Start a local Solana test validator and keep it running (it listens on `http://localhost:8899`), then run the tests
in a separate terminal:

```bash
# Terminal 1 — leave this running:
solana-test-validator

# Terminal 2 — from examples/basic_solana:
cargo test
```

> [!NOTE]
> The test must be able to reach the validator at `http://localhost:8899`. If `solana-test-validator` is not found, install the Solana CLI first (see the link above). `solana-test-validator` also writes a `test-ledger/` directory in the working directory; you can delete it between runs.

## Conclusion

In this example, you were able to:

* Deploy a canister on the ICP blockchain that can receive and send SOL.
* Connect the canister to the Solana Devnet.
* Send the canister some Devnet SOL.
* Use the canister to send SOL to another Solana account.
* Create a Solana nonce account and use the canister to send some SOL to another Solana account using durable nonces.
* Create an associated token account for an SPL token and use the canister to send some tokens to another Solana account.

## Security considerations and best practices

If you base your application on this example, we recommend you familiarize yourself with and adhere to
the [security best practices](https://docs.internetcomputer.org/guides/security/overview/) for developing on the
Internet Computer. This example may not implement all the best practices.

For example, the following aspects are particularly relevant for this app:

* [Certify query responses if they are relevant for security](https://docs.internetcomputer.org/guides/security/data-integrity-and-authenticity/#using-certified-variables-for-secure-queries),
  since the app offers a method to read balances, for example.
* [Use a decentralized governance system like SNS to make a canister have a decentralized controller](https://docs.internetcomputer.org/guides/security/canister-control/#use-a-governance-framework-such-as-the-sns-to-control-your-canisters),
  since decentralized control may be essential for canisters holding SOL on behalf of users.
