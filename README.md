[![Internet Computer portal](https://img.shields.io/badge/InternetComputer-grey?logo=internet%20computer&style=for-the-badge)](https://internetcomputer.org)
[![DFinity Forum](https://img.shields.io/badge/help-post%20on%20forum.dfinity.org-blue?style=for-the-badge)](https://forum.dfinity.org/t/sol-rpc-canister/41896)
[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg?logo=apache&style=for-the-badge)](LICENSE)

# SOL RPC canister

> [!IMPORTANT]  
> The SOL RPC canister and its associated libraries are under active development and are subject to change. Access to the repositories has been opened to allow for early feedback. Check back regularly for updates.
>
> Please share your feedback on the [developer forum](https://forum.dfinity.org/t/sol-rpc-canister/41896).


Interact with the [Solana](https://solana.com/) blockchain from the [Internet Computer](https://internetcomputer.org/).

## Table of Contents

* [Features](#features)
* [Usage](#usage)
    * [From the command line](#from-the-command-line)
    * [From within a Rust canister](#from-within-a-rust-canister)
* [Limitations](#limitations)
* [Reproducible build](#reproducible-build)
* [Related projects](#related-projects)
* [Contributing](#contributing)
* [License](#license)

## Features

* **No single point of failure**:  Each request will by default query 3 distinct Solana JSON-RPC providers and aggregate their results.
* **Configurable consensus strategy**: Choose how responses from multiple providers are aggregated depending on the needs of your application, e.g., 3-out-of-5 meaning that 5 providers will be queried and the overall response will be successful if at least 3 do agree (equality).
* **Pay directly in cycles**: No need to take care of API keys, each request can be paid for by attaching cycles.
* **Bring your own**: 
    * A Solana RPC method is not supported? There is an endpoint (`jsonRequest`) to send any JSON-RPC request.
    * Missing your favorite Solana JSON-RPC provider? You can specify your own providers (`RpcSources::Custom`).

## Usage

The SOL RPC canister runs on the [fiduciary subnet](https://internetcomputer.org/docs/building-apps/developing-canisters/deploy-specific-subnet#fiduciary-subnets) with the following principal: [`tghme-zyaaa-aaaar-qarca-cai`](https://dashboard.internetcomputer.org/canister/tghme-zyaaa-aaaar-qarca-cai).

Refer to the [Reproducible Build](#reproducible-build) section for information on how to verify the hash of the deployed WebAssembly module.

### From the command line

#### Prerequisites:

* [Install](https://internetcomputer.org/docs/building-apps/developer-tools/dev-tools-overview#dfx) `dfx`.
* [Cycles wallet](https://internetcomputer.org/docs/building-apps/canister-management/cycles-wallet) with some cycles to pay for requests.
* Commands are executed in [`canister/prod`](canister/prod).

#### Example with [`getSlot`](https://solana.com/de/docs/rpc/http/getslot)

To get the last `finalized` slot on Solana Mainnet

```bash
dfx canister call --ic sol_rpc --wallet $(dfx identity get-wallet --ic) --with-cycles 2B getSlot \
'
(
  variant { Default = variant { Mainnet } },
  opt record {
    responseConsensus = opt variant { Equality };
  },
  opt record { commitment = opt variant { finalized } },
)'
```

More examples are available [here](canister/scripts/examples.sh).

### From within a Rust Canister

#### Prerequisites:

* Add the `sol_rpc_client` library as a dependency in your `Cargo.toml`.

#### Example with [`getSlot`](https://solana.com/de/docs/rpc/http/getslot)

To get the last `finalized` slot on Solana Mainnet:

```rust,ignore
use sol_rpc_client::SolRpcClient;
use sol_rpc_types::{
    CommitmentLevel, ConsensusStrategy, GetSlotParams, RpcConfig, RpcSources, SolanaCluster,
};

let client = SolRpcClient::builder_for_ic()
    .with_rpc_sources(RpcSources::Default(SolanaCluster::Mainnet))
    .with_rpc_config(RpcConfig {
        response_consensus: Some(ConsensusStrategy::Equality),
        ..Default::default()
    })
    .build();

let slot = client
    .get_slot()
    .with_params(GetSlotParams {
        commitment: Some(CommitmentLevel::Finalized),
        ..Default::default()
    })
    .send()
    .await;
```

Full examples are available in the [examples](examples) folder and additional code snippets are also available in the [`sol_rpc_client`](libs/client/README.md) crate.

## Limitations

The SOL RPC canister reaches the Solana JSON-RPC providers using [HTTPS outcalls](https://internetcomputer.org/https-outcalls) and are therefore subject to the following limitations:
1. The contacted providers must support IPv6.
2. Some Solana RPC endpoint cannot be supported. This is the case for example for [`getLatestBlockhash`](https://solana.com/de/docs/rpc/http/getlatestblockhash).
   The reason is that an HTTPs outcalls involves an HTTP request from each node in the subnet and has therefore a latency in the order of a few seconds. 
   This can be problematic for endpoints with fast changing responses, such as [`getLatestBlockhash`](https://solana.com/de/docs/rpc/http/getlatestblockhash) (which changes roughly every 400ms),
   since in this case nodes will not be able to reach a consensus.
3. Note that in some cases, the use of a [response transformation](https://internetcomputer.org/docs/building-apps/network-features/using-http/https-outcalls/overview)
   to canonicalize the response seen by each node before doing consensus may alleviate the problem.
   For example, `getSlot` rounds by default the received slot by 20, therefore artificially increasing the slot time seen by each node to 8s to allow them reaching consensus with some significantly higher probability.
   The reason why such a canonicalization strategy does not work for [`getLatestBlockhash`](https://solana.com/de/docs/rpc/http/getlatestblockhash) is that the result is basically a random-looking string of fixed length.
4. There are therefore two options to send a transaction on Solana using the SOL RPC canister (see the [examples](examples))
   1. Use a [durable nonce](https://solana.com/de/developers/guides/advanced/introduction-to-durable-nonces) instead of a blockhash.
   2. Retrieve a recent blockhash by first retrieving a recent slot with `getSlot` and then getting the block (which includes the blockhash) with `getBlock`.

## Supported methods

The limitations described above imply that it is sometimes necessary to adapt a raw response from a Solana endpoint to increase the likelihood of nodes reaching consensus when querying that endpoint using [HTTPS outcalls](https://internetcomputer.org/https-outcalls).
The table below summarizes the supported endpoints and the necessary changes made to the response.

   | Solana method                                                                                   | Known limitations                                                           |
   |-------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------|
   | [`getAccountInfo`](https://solana.com/de/docs/rpc/http/getaccountinfo)                          | <ul><li>The field `context` is removed from the response</li></ul>          |
   | [`getBalance`](https://solana.com/de/docs/rpc/http/getbalance)                                  | <ul><li>The field `context` is removed from the response</li></ul>          |
   | [`getBlock`](https://solana.com/de/docs/rpc/http/getblock)                                      |                                                                             |
   | [`getRecenPrioritizationFees`](https://solana.com/de/docs/rpc/http/getrecentprioritizationfees) | <ul><li>Returns a subset of the response (configurable by caller)</li></ul> |
   | [`getSignatureStatuses`](https://solana.com/de/docs/rpc/http/getsignaturestatuses)              | <ul><li>The field `confirmations` is removed from the response</li></ul>    |
   | [`getSlot`](https://solana.com/de/docs/rpc/http/getslot)                                        | <ul><li>The result is rounded down (configurable by caller)</li></ul>       |
   | [`getTokenAccountBalance`](https://solana.com/de/docs/rpc/http/gettokenaccountbalance)          | <ul><li>The field `context` is removed from the response</li></ul>          |
   | [`getTransaction`](https://solana.com/de/docs/rpc/http/gettransaction)                          |                                                                             |
   | [`sendTransaction`](https://solana.com/de/docs/rpc/http/sendtransaction)                        |                                                                             |


## Reproducible Build

The SOL RPC canister supports [reproducible builds](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/test/reproducible-builds):

1. Ensure [Docker](https://www.docker.com/get-started/) is installed on your machine.
2. Run [`docker-build`](scripts/docker-build) in your terminal.
3. Run `sha256sum sol_rpc_canister.wasm.gz` on the generated file to view the SHA-256 hash.

In order to verify the latest SOL RPC Wasm file, please make sure to download the corresponding version of the source code from the latest GitHub release.

## Related Projects

* [`ic-solana`](https://github.com/mfactory-lab/ic-solana)

## Contributing

At this point we do not accept external contributions yet. External contributions will be accepted after the initial release.

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/licenses/Apache-2.0).