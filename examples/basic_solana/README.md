---
keywords: [ advanced, rust, solana, sol, integration, solana integration ]
---

# Basic Solana

## Overview

This tutorial will walk you through how to deploy a
sample [canister smart contract](TODO)
**that can send and receive SOL** on the Internet Computer.

## Architecture

This example internally leverages
the [threshold EdDSA](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/encryption/t-schnorr)
and [HTTPs outcalls](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/advanced-features/https-outcalls/https-outcalls-overview)
features of the Internet Computer.

For a deeper understanding of the ICP < > SOL integration, see
the [Solana integration overview](TODO).

## Prerequisites

* [x] Install the [IC SDK](https://internetcomputer.org/docs/current/developer-docs/setup/install/index.mdx).

## Step 1: Building and deploying sample code

### Clone the smart contract

To clone and build the smart contract in **Rust**:

```bash
git clone https://github.com/dfinity/sol-rpc-canister
cd examples/basic_solana
git submodule update --init --recursive
```

**If you are using MacOS, you'll need to install Homebrew and run `brew install llvm` to be able to compile the example.
**

### Acquire cycles to deploy

Deploying to the Internet Computer
requires [cycles](https://internetcomputer.org/docs/current/developer-docs/getting-started/tokens-and-cycles) (the
equivalent of "gas" on other blockchains).

### Deploy the smart contract to the Internet Computer

```bash
dfx deploy --ic basic_solana --argument '(opt record {solana_network = opt variant {Devnet}; ed25519_key_name = opt variant {TestKey1}})'
```

#### What this does

- `dfx deploy` tells the command line interface to `deploy` the smart contract
- `--ic` tells the command line to deploy the smart contract to the mainnet ICP blockchain
- `--argument (opt record {solana_network = opt variant {Devnet}; ed25519_key_name = opt variant {TestKey1}})`
  initializes the smart contract with the provided arguments:
    - `solana_network = opt variant {Devnet}`: the canister uses
      the [Solana Devnet](https://solana.com/docs/core/clusters)
      network.
    - `ed25519_key_name = opt variant {TestKey1}`: the canister uses a test key for signing via threshold EdDSA that is
      available on the ICP mainnet.
      See [signing messages](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/encryption/signing-messages#signing-messages-1)
      for more details.

If successful, you should see an output that looks like this:

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

In the output above, to see the Candid Web UI for your Solana canister, you would use the
URL `https://bd3sg-teaaa-aaaaa-qaaba-cai.raw.icp0.io/?id=<YOUR-CANISTER-ID>`. You should see the methods specified in
the Candid file `basic_solana.did`.

## Step 2: Generating a Solana account

A Solana account can be derived from an EdDSA public key. To derive a user's specific account, identified on the IC
by a principal, the canister uses its own threshold EdDSA public key to derive a new public key deterministically for
each requested principal. To retrieve your Solana account, you can call the `solana_account` method on the
previously deployed canister:

```shell
dfx canister --ic call basic_solana solana_account
```

This will return a Solana account such as `("2kqg1tEj59FNe3hSiLH88SySB9D7fUSArum6TP6iHFQY")` that is tied to your
principal. Your account will be different. You can view such accounts on any Solana block explorer such
as [Solana Explorer](https://explorer.solana.com/?cluster=devnet).

If you want to send some SOL to someone else, you can also use the above method to enquire about their Solana account
given their IC principal:

```shell
dfx canister --ic call basic_solana solana_account '(opt principal "hkroy-sm7vs-yyjs7-ekppe-qqnwx-hm4zf-n7ybs-titsi-k6e3k-ucuiu-uqe")'
```

This will return a different Solana address as the one above, such
as `("8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq")`.

## Step 3: Receiving SOL

Now that you have your Solana account, let us send some (Devnet) SOL to it:

1. Get some Devnet SOL if you don't have any. You can for example use [this faucet](https://faucet.solana.com/).
2. Send some Devnet SOL to the address you obtained in the previous step. You can use any Solana wallet to do so.

Once the transaction is confirmed, you'll be able to see it in your Solana account's balance, which should be visible
in a Solana block explorer,
e.g., https://explorer.solana.com/address/2kqg1tEj59FNe3hSiLH88SySB9D7fUSArum6TP6iHFQY?cluster=devnet.

## Step 4: Sending SOL

You can send ETH using the `send_sol` endpoint on your canister, specifying a Solana destination account and an amount
in the smallest unit (Lamport). For example, to send 1 Lamport to `8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq`, run
the
following command:

```shell
dfx canister --ic call basic_solana send_sol '("8HNiduWaBanrBv8c2pgGXZWnpKBdEYuQNHnspqto4yyq", 1)'
```

The `send_sol` endpoint sends SOL by executing the following steps:

1. Retrieving a recent blockhash. This is necessary because all Solana transactions must include a blockhash within the
   151 most recent stored hashes (which corresponds to about 60 to 90 seconds).
2. Estimating the current transaction fees. For simplicity, the current fees are hard-coded with a generous limit. A
   real world application would dynamically fetch the latest transaction fees, for example using the [
   `getRecentPrioritizationFees`](TODO)
   method in the [SOL-RPC canister](TODO).
3. Building a Solana transaction to send the specified amount to the given receiver's address.
4. Signing the Solana transaction using
   the [sign_with_schnorr API](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/signatures/signing-messages-t-schnorr).
5. Sending the signed transaction to the Solana network using the [`sendTransaction`](TODO) method in
   the [SOL-RPC canister](TODO).

The `send_sol` endpoint returns the transaction ID of the transaction sent to the Solana network, which can for example
be used
to track the transaction on a Solana blockchain explorer.

## Step 5: Sending SOL using durable nonces

TODO.

## Step 6: Sending USDC (using the Solana Token Library)

Send some USDC on Solana Devnet using the SPL. The USDC mint address on Devnet is
`4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`.

1. Create the sender and recipient Associated Token Accounts (ATA):
   ```shell
    dfx identity use sender
    dfx canister call basic_solana create_associated_token_account '("<TOKEN MINT ADDRESS`>")'
    dfx identity use recipient
    dfx canister call basic_solana create_associated_token_account '("<TOKEN MINT ADDRESS`>")'
    ```
2. Send some tokens to the sender ATA using e.g. [this faucet](https://faucet.circle.com/). To get the ATA, run:
   ```shell
    dfx identity use sender
    dfx canister call basic_solana associated_token_account '("<TOKEN MINT ADDRESS`>")'
    ```
3. Transfer some tokens from the sender to the recipient
   ```shell
    dfx identity use sender
    dfx canister call basic_solana send_spl_token '("<TOKEN MINT ADDRESS>", "<RECIPIENT SOLANA ADDRESS>", <AMOUNT>)'
    ```
4. Check out the transaction on [Solana Explorer](https://explorer.solana.com/?cluster=devnet). 

## Conclusion

In this tutorial, you were able to:

* Deploy a canister smart contract on the ICP blockchain that can receive and send SOL.
* Acquire cycles to deploy the canister to the ICP mainnet.
* Connect the canister to the Solana Devnet.
* Send the canister some Devnet SOL.
* Use the canister to send SOL to another Solana account.

Additional examples regarding the ICP < > SOL integration can be
found [here](TODO).

## Security considerations and best practices

If you base your application on this example, we recommend you familiarize yourself with and adhere to
the [security best practices](https://internetcomputer.org/docs/current/references/security/) for developing on the
Internet Computer. This example may not implement all the best practices.

For example, the following aspects are particularly relevant for this app:

* [Certify query responses if they are relevant for security](https://internetcomputer.org/docs/current/references/security/general-security-best-practices#certify-query-responses-if-they-are-relevant-for-security),
  since the app offers a method to read balances, for example.
* [Use a decentralized governance system like SNS to make a canister have a decentralized controller](https://internetcomputer.org/docs/current/references/security/rust-canister-development-security-best-practices#use-a-decentralized-governance-system-like-sns-to-make-a-canister-have-a-decentralized-controller),
  since decentralized control may be essential for canisters holding ETH on behalf of users.
