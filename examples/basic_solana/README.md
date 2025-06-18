---
keywords: [ advanced, chain fusion, integration, rust, sol, solana, solana integration, spl ]
---

# Basic Solana

## Overview

This tutorial will walk you through how to deploy a simple smart contract on the Internet Computer
(known as [canisters](https://internetcomputer.org/docs/building-apps/essentials/canisters)) **that can control digital
assets** on the Solana blockchain:
1. SOL, the native currency on Solana;
2. any other token (known as [SPL tokens](https://solana.com/docs/tokens)).

## Architecture

This example internally leverages
the [threshold EdDSA](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/encryption/t-schnorr)
and [HTTPs outcalls](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/advanced-features/https-outcalls/https-outcalls-overview)
features of the Internet Computer.

For a deeper understanding of the ICP <> SOL integration, see
the [chain fusion overview](https://internetcomputer.org/docs/building-apps/chain-fusion/solana/overview#sol-rpc-canister).

## Prerequisites

* [ ] Install the [IC SDK](https://internetcomputer.org/docs/current/developer-docs/setup/install/index.mdx) v0.27.0. If the IC SDK is already installed with an old version, install 0.27.0 with [`dfxvm`](https://internetcomputer.org/docs/building-apps/developer-tools/dev-tools-overview#dfxvm).
* [ ] Confirm the IC SDK has been installed with the correct version:
```shell
dfx --version
```

## Step 1: Building and deploying sample code

### Clone the smart contract

To clone and build the smart contract in **Rust**:

```bash
git clone https://github.com/dfinity/sol-rpc-canister
cd examples/basic_solana
git submodule update --init --recursive
```

**If you are using macOS, you'll need to install Homebrew and run `brew install llvm` to be able to
compile the example.**

### Acquire cycles to deploy

Deploying to the Internet Computer
requires [cycles](https://internetcomputer.org/docs/current/developer-docs/getting-started/tokens-and-cycles) (the
equivalent of "gas" on other blockchains).

### Deployment

#### Mainnet deployment

To deploy the Solana wallet smart contract to the ICP Mainnet, navigate to `examples/basic_solana/mainnet` and execute
the following command:

```bash
dfx deploy --ic
```

This deploys a wallet canister to the ICP Mainnet which is configured to interact with the Solana Devnet via the SOL RPC
canister at [`tghme-zyaaa-aaaar-qarca-cai`](https://dashboard.internetcomputer.org/canister/tghme-zyaaa-aaaar-qarca-cai).
Note that you will need to pay for your requests with cycles. If you do not have cycles available for testing, consider
running this example locally as described in the next section.

#### Local deployment

To deploy the Solana wallet smart contract locally, navigate to `examples/basic_solana/local` and execute the 
following commands:

```bash
dfx start --clean --background 
dfx deploy
./provision.sh
```

What this does:
- `dfx start --clean --background` starts a local instance of the ICP blockchain. 
- `dfx deploy` deploys a wallet canister as well as a SOL RPC canister, both locally. The wallet canister interacts with
  the Solana Devnet via the local SOL RPC canister.

**NOTE:** If running this example locally, you will need to skip the `--ic` flag in all subsequent `dfx` commands.

### Getting the canister ID

If the canister deployment is successful (whether on Mainnet of locally), you should see an output that looks like this:

```bash
Deploying: basic_solana
Building canisters...
...
Deployed canisters.
URLs:
Candid:
    basic_solana: https://bd3sg-teaaa-aaaaa-qaaba-cai.raw.icp0.io/?id=<YOUR-CANISTER-ID>
```

Your canister is live and ready to use! You can interact with it using either the command line or using the Candid UI,
which is the link you see in the output above.

In the output above, to see the Candid Web UI for your Solana canister, you would use the URL
`https://bd3sg-teaaa-aaaaa-qaaba-cai.raw.icp0.io/?id=<YOUR-CANISTER-ID>`. You should see the methods specified in the
Candid file `basic_solana.did`.

## Step 2: Generating a Solana account

A Solana account can be derived from an EdDSA public key. To derive a user's specific account, identified on the IC by a
principal, the canister uses its own threshold EdDSA public key to derive a new public key deterministically for each
requested principal. To retrieve your Solana account, you can call the `solana_account` method on the previously
deployed canister:

```shell
dfx canister --ic call basic_solana solana_account
```

This will return a Solana account such as `("2kqg1tEj59FNe3hSiLH88SySB9D7fUSArum6TP6iHFQY")` that is tied to your
principal. Your account will be different. You can view such accounts on any Solana explorer such
as [Solana Explorer](https://explorer.solana.com/?cluster=devnet).

If you want to send some SOL to someone else, you can also use the above method to enquire about their Solana account
given their IC principal:

```shell
dfx canister --ic call basic_solana solana_account '(opt principal "hkroy-sm7vs-yyjs7-ekppe-qqnwx-hm4zf-n7ybs-titsi-k6e3k-ucuiu-uqe")'
```

This will return a different Solana address as the one above, such as
`("8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq")`.

## Step 3: Receiving SOL

Now that you have your Solana account, let us send some (Devnet) SOL to it:

1. Get some Devnet SOL if you don't have any. You can for example use [this faucet](https://faucet.solana.com/).
2. Send some Devnet SOL to the address you obtained in the previous step. You can use any Solana wallet to do so.

Once the transaction is confirmed, you'll be able to see it in your Solana account's balance, which should be visible in
a Solana explorer, e.g. https://explorer.solana.com/address/2kqg1tEj59FNe3hSiLH88SySB9D7fUSArum6TP6iHFQY?cluster=devnet.

## Step 4: Sending SOL

You can send SOL using the `send_sol` endpoint on your canister, specifying a Solana destination account and an amount
in the smallest unit (Lamport). For example, to send 1 Lamport to `8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq`, run
the following command:

```shell
dfx canister --ic call basic_solana send_sol '(null, "8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq", 1)'
```

The `send_sol` endpoint sends SOL by executing the following steps:

1. Retrieving a [recent blockhash](https://solana.com/docs/core/transactions#recent-blockhash). This is necessary
   because all Solana transactions must include a blockhash within the 151 most recent stored hashes (which corresponds
   to about 60 to 90 seconds).
2. Building a Solana [transaction](https://solana.com/docs/core/transactions) that includes a single instruction to
   transfer the specified amount from the sender's address to the given receiver's address, as well as the recent
   blockhash.
3. Signing the Solana transaction using
   the [threshold Ed25519 API](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/signing-messages-t-schnorr).
4. Sending the signed transaction to the Solana network using the `sendTransaction` method in
   the [SOL RPC canister](https://github.com/dfinity/sol-rpc-canister).

The `send_sol` endpoint returns the transaction ID of the transaction sent to the Solana network, which can for example
be used to track the transaction on a Solana blockchain explorer.

## Step 5: Sending SOL using durable nonces

[Durable nonces](https://solana.com/developers/guides/advanced/introduction-to-durable-nonces) can be used instead of a
recent blockhash when constructing a Solana transaction. This can be useful for example when signing a transaction in
advance before sending it out.

In order to use durable nonces, you must first create a nonce account controlled by your Solana account. The nonce
account contains the current value of the durable nonce. To create a nonce account controlled by your Solana account,
run the following command:

```shell
dfx canister --ic call basic_solana create_nonce_account
```

You can inspect the created nonce account and get the current durable nonce value in a Solana explorer. You can also
fetch the current value of the durable nonce by running the following command:

```shell
dfx canister --ic call basic_solana get_nonce
```

To send some SOL using a durable nonce, you can run the following command:

```shell
dfx canister --ic call basic_solana send_sol_with_durable_nonce '(null, "8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq", 1)'
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
   the [threshold Ed25519 API](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/signing-messages-t-schnorr).
4. Sending the signed transaction to the Solana network using the `sendTransaction` method in
   the [SOL RPC canister](https://github.com/dfinity/sol-rpc-canister).

The `send_sol_with_durable_nonce` endpoint returns the transaction ID of the transaction sent to the Solana network. You
can also verify (either in a Solana explorer or using the `get_nonce` endpoint) that the nonce value stored in the
account has changed after calling this endpoint.

## Step 6: Sending Solana Program Library (SPL) tokens

We will now be sending some SPL tokens on Solana Devnet. The instructions below work for any SPL token. You may for
example use the USDC token whose mint account on Devnet is `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`.

You first need to create [Associated Token Accounts (ATA)](https://spl.solana.com/associated-token-account) for the
sender and recipient accounts if they do not exist yet. An ATA is
a [Program Derived Address (PDA)](https://solana.com/docs/core/pda) derived from a Solana account using the token mint
account. An ATA is needed for each type of SPL token held by a Solana account. To create the ATAs for the sender and
recipient, you can run the following commands:

```bash
dfx identity use sender
dfx canister call basic_solana create_associated_token_account '(null, "<TOKEN MINT ADDRESS`>")'
dfx identity use recipient
dfx canister call basic_solana create_associated_token_account '(null, "<TOKEN MINT ADDRESS`>")'
```

or

```bash
dfx canister call basic_solana create_associated_token_account '(opt principal "<SENDER PRINCIPAL>", "<TOKEN MINT ADDRESS`>")'
dfx canister call basic_solana create_associated_token_account '(opt principal "<RECIPIENT PRINCIPAL>", "<TOKEN MINT ADDRESS`>")'
```

This works by sending transactions that instruct the
Solana [Associated Token Account Program](https://spl.solana.com/associated-token-account) to create and initialize
these accounts. You can now inspect the sender and recipient accounts on a Solana explorer and confirm that you can see
a balance of 0 for the corresponding SPL token.

To send some tokens from the sender to the receiver, you will need to obtain some tokens on the sender account (using
e.g. [this faucet](https://faucet.circle.com/) for USDC). To do this, you will need the ATA address of the sender. You
can for example get it by running the following command:

```bash
dfx identity use sender
dfx canister call basic_solana associated_token_account '(null, "<TOKEN MINT ADDRESS`>")'
```

To transfer some tokens from the sender to the recipient, you can run the following command:

```bash
dfx identity use sender
dfx canister call basic_solana send_spl_token '(null, "<TOKEN MINT ADDRESS>", "<RECIPIENT SOLANA ADDRESS>", <AMOUNT>)'
```

The `send_spl_token` endpoint works similarly to the `send_sol` endpoint, but creates a transaction with the sender and
recipient ATAs instead of their account addresses. You can also inspect the resulting transaction on a Solana explorer,
and verify that the associated token balances were updated accordingly. You can also check the updated token balances by
running the following commands:

```bash
dfx canister call basic_solana get_spl_token_balance '(opt principal "<SENDER PRINCIPAL>", "<TOKEN MINT ADDRESS`>")'
dfx canister call basic_solana get_spl_token_balance '(opt principal "<RECIPIENT PRINCIPAL>", "<TOKEN MINT ADDRESS`>")'
```

## Conclusion

In this tutorial, you were able to:

* Deploy a canister smart contract on the ICP blockchain that can receive and send SOL.
* Acquire cycles to deploy the canister to the ICP Mainnet.
* Connect the canister to the Solana Devnet.
* Send the canister some Devnet SOL.
* Use the canister to send SOL to another Solana account.
* Create a Solana nonce account and use the canister to send some SOL to another Solana account using durable nonces.
* Create an associated token account for an SPL token use the canister to send some tokens to another Solana account.

Additional examples regarding the ICP <> SOL integration can be
found [here](https://github.com/dfinity/sol-rpc-canister/tree/main/examples).

## Security considerations and best practices

If you base your application on this example, we recommend you familiarize yourself with and adhere to
the [security best practices](https://internetcomputer.org/docs/current/references/security/) for developing on the
Internet Computer. This example may not implement all the best practices.

For example, the following aspects are particularly relevant for this app:

* [Certify query responses if they are relevant for security](https://internetcomputer.org/docs/current/references/security/general-security-best-practices#certify-query-responses-if-they-are-relevant-for-security),
  since the app offers a method to read balances, for example.
* [Use a decentralized governance system like SNS to make a canister have a decentralized controller](https://internetcomputer.org/docs/current/references/security/rust-canister-development-security-best-practices#use-a-decentralized-governance-system-like-sns-to-make-a-canister-have-a-decentralized-controller),
  since decentralized control may be essential for canisters holding SOL on behalf of users.
