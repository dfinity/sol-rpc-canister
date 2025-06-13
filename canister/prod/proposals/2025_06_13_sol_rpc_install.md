# Proposal to install the SOL RPC canister

Repository: `https://github.com/dfinity/sol-rpc-canister.git`

Git hash: `e3e416aea9292deaa1a757537bfef76948890eb6`

New compressed Wasm hash: `3e13d284ad221f716f4665ef9d7d9157ba192d6717c14917d2993d480ee70177`

Install args hash: `926c1fb80684f1950f2a19ca3d2425e0c4af6597340ec36360299f41b5824147`

Target canister: `tghme-zyaaa-aaaar-qarca-cai`

---

## Motivation
This proposal installs the SOL RPC canister at version [v1.0.0](https://github.com/dfinity/sol-rpc-canister/releases/tag/v1.0.0) to the NNS-controlled canister ID [`tghme-zyaaa-aaaar-qarca-cai`](https://dashboard.internetcomputer.org/canister/tghme-zyaaa-aaaar-qarca-cai) on the fiduciary subnet [`pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae`](https://dashboard.internetcomputer.org/subnet/pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae).

## Install args

```
git fetch
git checkout e3e416aea9292deaa1a757537bfef76948890eb6
didc encode -d canister/sol_rpc_canister.did -t '(InstallArgs)' '( record { manageApiKeys = opt vec { principal "mf7xa-laaaa-aaaar-qaaaa-cai" }; overrideProvider = null; logFilter = opt variant { ShowAll }; numSubnetNodes = opt 34; mode = opt variant { Normal };  } )' | xxd -r -p | sha256sum
```

* The principal `mf7xa-laaaa-aaaar-qaaaa-cai` is a DFINITY-controlled wallet that can manage API keys.
* `overrideProvider` is set to `null`, meaning that the default RPC provider URLs and HTTP headers are not overridden.
* The `logFilter` is set to `ShowAll`, meaning that all logs will be visible.
* `numSubnetNodes` is set to `34`, the number of nodes in the fiduciary subnet `pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae`.
* The `mode` is set to `Normal`, requiring callers to attach cycles to use the SOL RPC canister.

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout e3e416aea9292deaa1a757537bfef76948890eb6
"./scripts/docker-build"
sha256sum ./wasms/sol_rpc_canister.wasm.gz
```