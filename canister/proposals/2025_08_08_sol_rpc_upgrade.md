# Proposal to upgrade the SOL RPC canister

Repository: `https://github.com/dfinity/sol-rpc-canister.git`

Git hash: `77e4d2fa2da424d5538e624ff2fdf6cb33b55447`

New compressed Wasm hash: `b51932ccb340013beddd506d9326edc277b64f951593be80a169cce63aa511e6`

Upgrade args hash: `e04af557f2ed69b3396cb58bf6623e7fb645a275385f68e90cd880169a328b41`

Target canister: `tghme-zyaaa-aaaar-qarca-cai`

Previous SOL RPC proposal: https://dashboard.internetcomputer.org/proposal/136985

---

## Motivation

Upgrade the SOL RPC canister to the latest version [v1.1.0](https://github.com/dfinity/sol-rpc-canister/releases/tag/sol_rpc_canister-v1.1.0),
which includes in particular the following changes:
* Add more metrics.
* Require HTTP outcall base fee for update calls.
* Select supported providers based on successful responses.

See the Gihub release [v1.1.0](https://github.com/dfinity/sol-rpc-canister/releases/tag/sol_rpc_canister-v1.1.0) for more details.

## Release Notes

```
git log --format='%C(auto) %h %s' e3e416aea9292deaa1a757537bfef76948890eb6..77e4d2fa2da424d5538e624ff2fdf6cb33b55447 --
77e4d2f chore: release (#201)
ddc1212 docs: remove warning in top-level README (#199)
3cae012 chore: do not record metrics for requests with insufficient cycles (#184)
4d142a5 refactor: require HTTP outcall base fee (#185)
7483ab1 build: update dependencies (#196)
b9725ec feat!: add `try_send` method to SOL RPC client (#187)
7ed9669 docs: improvements to SOL RPC docs (#158)
8e004fd fix: remove unneeded `rename` annotations (#186)
715f278 feat: Select supported providers based on successful responses (#183)
2c7d70b build: update dependencies (#182)
5511e7e build: release with multi-tags (#174)
5ac94de chore: add optional `cost_units` to `TransactionStatusMeta` (#180)
1c3bc56 chore: add more metrics (#144)
10577e6 chore: add `Custom` Solana network to `basic_solana` canister (#171)
b30ad3e docs: add deployment instructions (#173)
ff0f747 chore: revert `sol_rpc_client` bump (#178)
6b8e60c build: migrate dependencies to `solana-sdk` repository (#55)
72473a7 docs: improve docs for `InstallArgs` (#172)
07c1fc2 docs: add build requirements to READMEs and rustdoc  (#169)
9cee54b docs: more links (#168)
3fd4f82 chore: bump `sol_rpc_client` to `v1.0.1` (#164)
d5e9a2a docs: enable `ed25519` feature in docs (#162)
0a084f4 docs: improve `basic_solana` Candid documentation (#161)
83ab43d fix: use correct fee for t-sig with local development key (#160)
bcd3231 chore: use build script for `basic_solana` deployment (#156)
a26fe79 build: add `Cargo.toml` linting to CI pipeline (#155)
bc13db6 test: add `basic_solana` deployment tests (#150)
2ff16bf fix: change `nat16` to `nat32` in examples (#151)
a791239 docs: add support for running `basic_solana` locally and on mainnet (#91)
293ed01 test: integration test for fetching metrics (#143)
eb4e630 chore: install SOL RPC canister at v1.0.0 (#148)
 ```

## Upgrade args

```
git fetch
git checkout 77e4d2fa2da424d5538e624ff2fdf6cb33b55447
didc encode -d canister/sol_rpc_canister.did -t '(InstallArgs)' '(record {})' | xxd -r -p | sha256sum
```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout 77e4d2fa2da424d5538e624ff2fdf6cb33b55447
"./scripts/docker-build"
sha256sum ./wasms/sol_rpc_canister.wasm.gz
```