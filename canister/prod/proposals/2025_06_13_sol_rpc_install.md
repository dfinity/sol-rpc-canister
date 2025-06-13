# Proposal to install the SOL RPC canister

Repository: `https://github.com/dfinity/sol-rpc-canister.git`

Git hash: `e3e416aea9292deaa1a757537bfef76948890eb6`

New compressed Wasm hash: `3e13d284ad221f716f4665ef9d7d9157ba192d6717c14917d2993d480ee70177`

Install args hash: `aaa3329e22ebfe9e1335a33515217e7cd0bed9789b0a0326888570e83a423218`

Target canister: `tghme-zyaaa-aaaar-qarca-cai`

---

## Motivation
This proposal installs the SOL RPC canister to the NNS-controlled canister ID `tghme-zyaaa-aaaar-qarca-cai` on subnet `pzp6e-ekpqk-3c5x7-2h6so-njoeq-mt45d-h3h6c-q3mxf-vpeq5-fk5o7-yae`.

## Install args

```
git fetch
git checkout e3e416aea9292deaa1a757537bfef76948890eb6
didc encode -d canister/sol_rpc_canister.did -t '(InstallArgs)' '( record { manageApiKeys = opt vec { principal "mf7xa-laaaa-aaaar-qaaaa-cai" }; overrideProvider = null; logFilter = null; numSubnetNodes = null; mode = null;  } )' | xxd -r -p | sha256sum
```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout e3e416aea9292deaa1a757537bfef76948890eb6
"./scripts/docker-build"
sha256sum ./wasms/sol_rpc_canister.wasm.gz
```