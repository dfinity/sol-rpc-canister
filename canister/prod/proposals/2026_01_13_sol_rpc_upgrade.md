# Proposal to upgrade the SOL RPC canister

Repository: `https://github.com/dfinity/sol-rpc-canister.git`

Git hash: `9402b1ebb4fb14bab05b5bf5498894b9aea84198`

New compressed Wasm hash: `1733b1e5a2c14f9488b5314d6c0d189669b3841b2d360a4f88a554216c423115`

Upgrade args hash: `e04af557f2ed69b3396cb58bf6623e7fb645a275385f68e90cd880169a328b41`

Target canister: `tghme-zyaaa-aaaar-qarca-cai`

Previous SOL RPC proposal: https://dashboard.internetcomputer.org/proposal/138474

---

## Motivation
TODO: THIS MUST BE FILLED OUT


## Release Notes

```
git log --format='%C(auto) %h %s' 46f5c95e6d0200675d10e905fd20fbf8d4cf8968..9402b1ebb4fb14bab05b5bf5498894b9aea84198 --
9402b1e chore: release (#259)
e8d8003 fix: explicitly list approved files in `BOT_APPROVED_FILES` (#262)
08b074e ci: add `BOT_APPROVED_FILES` (#261)
5976865 chore: upgrade dependencies (#260)
c4612ad fix!: calculate default request cost before sending (#256)
5857330 fix: remove `basic_solana` override from `release-plz.toml` (#258)
b44ba6a refactor: use `ic-pocket-canister-runtime` mocking infrastructure (#252)
50c0c60 refactor: use canister runtime crates (#248)
69d633c chore!: bump `ic-cdk` to v0.19.0 (#251)
d7e640e ci: queue end-to-end tests (#257)
74554de chore: update `CODEOWNERS` file (#255)
f8e0b56 refactor: use `serde_tuple` crate (#254)
532bdcc refactor: standalone `basic_solana` for ICP Ninja (#249)
026ecd0 chore: add comments to `hidden_endpoints` file (#250)
dcbb112 test: integration tests that RPC config is respected (#237)
66611b2 ci: check canister endpoints with `ic-wasm` (#238)
f7d6f8c fix: do not ignore `response_size_estimate` for `getBlock` (#236)
86c2d2c chore: handle `max_response_bytes` is exceeded error in metrics (#235)
476eaff chore: proposal to upgrade to v1.2.0 (#226)
1804cb5 ci: re-enable `check-ninja-cargo-toml` in CI pipeline (#223)
a273bab docs: expand `basic_solana` tip on local usage from ICP Ninja (#218)
a6ad368 chore: revert to symlink for `basic_solana` Ninja deployment (#222)
b8dd250 test: local ICP Ninja example deployment (#217)
 ```

## Upgrade args

```
git fetch
git checkout 9402b1ebb4fb14bab05b5bf5498894b9aea84198
didc encode -d canister/sol_rpc_canister.did -t '(InstallArgs)' '(record {})' | xxd -r -p | sha256sum
```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout 9402b1ebb4fb14bab05b5bf5498894b9aea84198
"./scripts/docker-build"
sha256sum ./wasms/sol_rpc_canister.wasm.gz
```