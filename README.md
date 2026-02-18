[![Internet Computer portal](https://img.shields.io/badge/InternetComputer-grey?logo=internet%20computer&style=for-the-badge)](https://internetcomputer.org)
[![DFinity Forum](https://img.shields.io/badge/help-post%20on%20forum.dfinity.org-blue?style=for-the-badge)](https://forum.dfinity.org/t/sol-rpc-canister/41896)
[![GitHub license](https://img.shields.io/badge/license-Apache%202.0-blue.svg?logo=apache&style=for-the-badge)](LICENSE)

# SOL RPC canister

Interact with the [Solana](https://solana.com/) blockchain from the [Internet Computer](https://internetcomputer.org/).

## Table of Contents

* [Features](#features)
* [Usage](#usage)
    * [From the command line](#from-the-command-line)
    * [From within a Rust canister](#from-within-a-rust-canister)
* [Deployment](#deployment)
  * [Deployment to the IC](#deployment-to-the-ic)
  * [Local deployment](#local-deployment)
  * [Deploying from ICP Ninja](#deploying-from-icp-ninja)
* [Limitations](#limitations)
* [Supported Methods](#supported-methods)
* [Supported Solana JSON-RPC Providers](#supported-solana-json-rpc-providers)
* [Reproducible build](#reproducible-build)
* [Learn More](#learn-more)
* [Related projects](#related-projects)
* [Contributing](#contributing)
* [Releasing](#releasing)
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

* [Install](https://internetcomputer.org/docs/building-apps/developer-tools/dev-tools-overview#icp-cli) `icp-cli` (minimum version: v0.1.0).
* [Cycles wallet](https://internetcomputer.org/docs/building-apps/canister-management/cycles-wallet) with some cycles to pay for requests.
* Commands are executed in [`canister/prod`](canister/prod).

#### Example with [`getSlot`](https://solana.com/de/docs/rpc/http/getslot)

To get the last `finalized` slot on Solana Mainnet

```bash
icp canister call --network ic sol_rpc --wallet $(icp identity get-wallet --network ic) --with-cycles 2B getSlot \
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

* Add the `sol_rpc_client` and `sol_rpc_types` libraries as dependencies in your `Cargo.toml`.
* Follow the steps outlined [here](libs/client/README.md#build-requirements) to ensure your code compiles.
* If you are running the example locally, follow the instructions [here](README.md#local-deployment) to deploy a local instance of the SOL RPC canister.

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

## Deployment

### Deployment to the IC

> [!TIP]
> When deploying your own instance of the SOL RPC canister, you will need to provide and manage your own API keys for the Solana RPC providers. You can provision these keys with the `updateApiKeys` canister endpoint.

To deploy your own instance of the SOL RPC canister to the IC Mainnet, first add the following to your `icp.yaml`:

```yaml
canisters:
  - name: sol_rpc
    build:
      steps:
        - type: remote
          candid: "https://github.com/dfinity/sol-rpc-canister/releases/latest/download/sol_rpc_canister.did"
          wasm: "https://github.com/dfinity/sol-rpc-canister/releases/latest/download/sol_rpc_canister.wasm.gz"
    init_args:
      sol_rpc: "( record {} )"

networks:
  - name: ic
    mode: connected
    url: https://icp-api.io

environments:
  - name: prod
    network: ic
    canisters: [sol_rpc]
```

You can also specify your own `init_args` to configure the SOL RPC canister's behaviour. For this, refer to the [Candid interface](canister/sol_rpc_canister.did) specification.

Finally, run the following command (from the directory containing your `icp.yaml`) to deploy the canister on the IC:

```sh
icp deploy --environment prod
```

### Local deployment

> [!IMPORTANT]
> Deploying the SOL RPC canister locally hides some important differences compared to deploying on the ICP Mainnet. Always test your solution on the ICP Mainnet before considering it production-ready. The following behaviors are possible in local environments but **not supported** on Mainnet:
> - **IPv4 HTTP outcalls:** Local development environments allow HTTP requests over IPv4, but the ICP Mainnet only supports IPv6 for HTTP outcalls. For example, Solana Foundation [public RPC endpoints](https://solana.com/docs/references/clusters#solana-public-rpc-endpoints), which are support only IPv4, will work locally but not on Mainnet.
> - **Single-replica behavior:** Local deployments run on a single replica, while Mainnet uses a replicated, consensus-based model. This can cause calls that work locally to fail on Mainnet due to consensus issues. For instance, calls to [`getLatestBlockhash`](https://solana.com/docs/rpc/http/getlatestblockhash) may succeed locally but fail on Mainnet because Solana’s fast block times can cause discrepancies between replicas during validation.

To deploy a local instance of the SOL RPC canister, first add the following to your `icp.yaml` config file:

```yaml
canisters:
  - name: sol_rpc
    build:
      steps:
        - type: remote
          candid: "https://github.com/dfinity/sol-rpc-canister/releases/latest/download/sol_rpc_canister.did"
          wasm: "https://github.com/dfinity/sol-rpc-canister/releases/latest/download/sol_rpc_canister.wasm.gz"
    init_args:
      sol_rpc: "( record {} )"

networks:
  - name: local
    mode: managed
    url: http://localhost:4943

environments:
  - name: local
    network: local
    canisters: [sol_rpc]
```

You can also specify your own `init_args` to configure the SOL RPC canister's behaviour. For this, refer to the [Candid interface](canister/sol_rpc_canister.did) specification.

Finally, run the following commands (from the directory containing your `icp.yaml`) to deploy the canister in your local environment:

```sh
# Start the local replica
icp network start --background

# Locally deploy the `sol_rpc` canister
icp deploy --environment local
```

### Deploying from ICP Ninja

To deploy the SOL RPC canister together with an example Solana wallet smart contract using ICP Ninja, click on the following link:

[![](https://icp.ninja/assets/open.svg)](https://icp.ninja/editor?g=https://github.com/dfinity/sol-rpc-canister/tree/main/examples/basic_solana/ninja)

> [!TIP]
> If you download the project from ICP Ninja to deploy it locally, you will need to change the `init_args` for the `basic_solana` canister. Specifically, you will need to change `ed25519_key_name = opt variant { MainnetTestKey1 }` to `ed25519_key_name = opt variant { LocalDevelopment }`. To learn more about the initialization arguments, see the `InitArg` type in [`basic_solana.did`](basic_solana.did).

## Limitations

The SOL RPC canister reaches the Solana JSON-RPC providers using [HTTPS outcalls](https://internetcomputer.org/https-outcalls) and are therefore subject to the following limitations:
1. The contacted providers must support IPv6.
2. Some Solana RPC endpoint cannot be supported. This is the case for example for [`getLatestBlockhash`](https://solana.com/de/docs/rpc/http/getlatestblockhash).
   The reason is that an HTTPs outcalls involves an HTTP request from each node in the subnet and has therefore a latency in the order of a few seconds. 
   This can be problematic for endpoints with fast changing responses, such as [`getLatestBlockhash`](https://solana.com/de/docs/rpc/http/getlatestblockhash) (which changes roughly every 400ms),
   since in this case nodes will not be able to reach a consensus.
3. Note that in some cases, the use of a [response transformation](https://internetcomputer.org/docs/building-apps/network-features/using-http/https-outcalls/overview)
   to canonicalize the response seen by each node before doing consensus may alleviate the problem. 
   The exact transform used depends on the Solana method being queried. See the section on [Supported methods](#supported-methods) for more details.
   For example, `getSlot` rounds by default the received slot by 20 (configurable by the caller), therefore artificially increasing the slot time seen by each node to 8s to allow them reaching consensus with some significantly higher probability.
   The reason why such a canonicalization strategy does not work for [`getLatestBlockhash`](https://solana.com/de/docs/rpc/http/getlatestblockhash) is that the result is basically a random-looking string of fixed length.
4. There are therefore two options to send a transaction on Solana using the SOL RPC canister (see the [examples](examples))
   1. Use a [durable nonce](https://solana.com/de/developers/guides/advanced/introduction-to-durable-nonces) instead of a blockhash.
   2. Retrieve a recent blockhash by first retrieving a recent slot with `getSlot` and then getting the block (which includes the blockhash) with `getBlock`.

## Supported methods

The limitations described above imply that it is sometimes necessary to adapt a raw response from a Solana endpoint to increase the likelihood of nodes reaching consensus when querying that endpoint using [HTTPS outcalls](https://internetcomputer.org/https-outcalls).
The table below summarizes the supported endpoints and the necessary changes (if any) made to the response indicated as follows:
* :white_check_mark: no changes are made to the raw response (excepted for JSON canonicalization).
* :scissors: one or several fields are either not supported in the request parameters, or removed from the raw response.
* :hammer_and_wrench: the raw response is more heavily transformed (e.g. rounding, subset, etc.).

| Solana method                                                                                   | Support              | Known limitations                                                                                                                                                                                                                                                                                       |
|-------------------------------------------------------------------------------------------------|----------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| [`getAccountInfo`](https://solana.com/de/docs/rpc/http/getaccountinfo)                          | :scissors:           | <ul><li>The field `context` is removed from the response</li></ul>                                                                                                                                                                                                                                      |
| [`getBalance`](https://solana.com/de/docs/rpc/http/getbalance)                                  | :scissors:           | <ul><li>The field `context` is removed from the response</li></ul>                                                                                                                                                                                                                                      |
| [`getBlock`](https://solana.com/de/docs/rpc/http/getblock)                                      | :scissors: | <ul><li>Only the `signatures` and `none` values for the `transactionDetails` request parameter are supported. If not specified, the default value is `none`.</li></ul><ul><li>The `encoding` request parameter is not supported.</li></ul> |
| [`getRecenPrioritizationFees`](https://solana.com/de/docs/rpc/http/getrecentprioritizationfees) | :hammer_and_wrench:  | <ul><li>Returns a subset of the response (configurable by caller)</li></ul>                                                                                                                                                                                                                             |
| [`getSignaturesForAddress`](https://solana.com/de/docs/rpc/http/getsignaturesforaddress)        | :white_check_mark:   | <ul><li>Use the field `before` to have idempotent responses</li></ul>                                                                                                                                                                                                                                   |
| [`getSignatureStatuses`](https://solana.com/de/docs/rpc/http/getsignaturestatuses)              | :scissors:           | <ul><li>The field `confirmations` is removed from the response</li></ul><ul><li>The field `context` is removed from the response</li></ul>                                                                                                                                                              |
| [`getSlot`](https://solana.com/de/docs/rpc/http/getslot)                                        | :hammer_and_wrench:  | <ul><li>The result is rounded down (configurable by caller)</li></ul>                                                                                                                                                                                                                                   |
| [`getTokenAccountBalance`](https://solana.com/de/docs/rpc/http/gettokenaccountbalance)          | :scissors:           | <ul><li>The field `context` is removed from the response</li></ul>                                                                                                                                                                                                                                      |
| [`getTransaction`](https://solana.com/de/docs/rpc/http/gettransaction)                          | :scissors: | <ul><li>Only the `base64` and `base58` values for the `encoding` request parameter are supported.</li></ul>                                                                                                                                                                                             |
| [`sendTransaction`](https://solana.com/de/docs/rpc/http/sendtransaction)                        | :white_check_mark:   |                                                                                                                                                                                                                                                                                                         |


## Supported Solana JSON-RPC Providers

| Provider                              | Solana Mainnet     | Solana Devnet      |
|---------------------------------------|--------------------|--------------------|
| [Alchemy](https://www.alchemy.com/)   | :white_check_mark: | :white_check_mark: |
| [Ankr](https://www.ankr.com/)         | :white_check_mark: | :white_check_mark: |
| [Chainstack](https://chainstack.com/) | :white_check_mark: | :white_check_mark: |
| [dRPC](https://drpc.org/)             | :white_check_mark: | :white_check_mark: |
| [Helius](https://www.helius.dev/)     | :white_check_mark: | :white_check_mark: |
| [PublicNode](https://publicnode.com/) | :white_check_mark: | :x:                |

## Reproducible Build

The SOL RPC canister supports [reproducible builds](https://internetcomputer.org/docs/current/developer-docs/smart-contracts/test/reproducible-builds):

1. Ensure [Docker](https://www.docker.com/get-started/) is installed on your machine.
2. Run [`docker-build`](scripts/docker-build) in your terminal.
3. Run `sha256sum sol_rpc_canister.wasm.gz` on the generated file to view the SHA-256 hash.

In order to verify the latest SOL RPC Wasm file, please make sure to download the corresponding version of the source code from the latest GitHub release.

## Learn More

* :movie_camera: [Demo](https://youtu.be/CpxQqp6CxoY?feature=shared) that runs through most parts of the [basic_solana](examples/basic_solana) example.
* :newspaper: Blog post [ICP Reaches the Shores of Solana](https://medium.com/dfinity/icp-reaches-the-shores-of-solana-0f373a886dce).
* :loudspeaker: [Forum post](https://forum.dfinity.org/t/sol-rpc-canister/41896) on the developer forum.

## Related Projects

* [`ic-solana`](https://github.com/mfactory-lab/ic-solana)

## Contributing

At this point we do not accept external contributions yet. External contributions will be accepted after the initial release.

## Releasing

1. Run the [`Release`](https://github.com/dfinity/sol-rpc-canister/actions/workflows/release.yml) workflow by clicking on `Run workflow`. The branch to use to run the workflow is typically `main`.
2. This will open a `Draft PR` with the label `release`. 
   1. Adapt the changelogs as needed.
   2. Go through the usual review process and merge when satisfied.
3. Run the [`Publish`](https://github.com/dfinity/sol-rpc-canister/actions/workflows/publish.yml) workflow by clicking on `Run workflow`. The branch to use to run the workflow is typically `main`. The job will do the following:
   1. Create Git tags.
   2. Publish crates on crates.io.
   3. Create a Github pre-release.

## License

This project is licensed under the [Apache License 2.0](https://opensource.org/licenses/Apache-2.0).
