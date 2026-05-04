# Proposal to upgrade the SOL RPC canister

Repository: `https://github.com/dfinity/sol-rpc-canister.git`

Git hash: `46f5c95e6d0200675d10e905fd20fbf8d4cf8968`

New compressed Wasm hash: `cbdebc79037029cdc2fcb11eb67ffc25791dcfc226923fba06557b171b406e6c`

Upgrade args hash: `e04af557f2ed69b3396cb58bf6623e7fb645a275385f68e90cd880169a328b41`

Target canister: `tghme-zyaaa-aaaar-qarca-cai`

Previous SOL RPC proposal: https://dashboard.internetcomputer.org/proposal/137793

---

## Motivation
Upgrade the SOL RPC canister to the latest version [v1.2.0](https://github.com/dfinity/sol-rpc-canister/releases/tag/sol_rpc_canister-v1.2.0) which replaces forked `solana-*` crates by the corresponding ones in the `solana_sdk` and `agave` repositories version 3.0.0.

See the Gihub release [v1.2.0](https://github.com/dfinity/sol-rpc-canister/releases/tag/sol_rpc_canister-v1.2.0) for more details.

## Release Notes

```
git log --format='%C(auto) %h %s' 77e4d2fa2da424d5538e624ff2fdf6cb33b55447..46f5c95e6d0200675d10e905fd20fbf8d4cf8968 --
46f5c95 ci: revert release-plz action to upstream repository (#221)
9b28e09 chore: release (#219)
a7b6fc9 fix: release pipeline (#215)
37729ba chore: use `spl-*` crates (#212)
f141f90 build!: replace forked `solana-*` crates by latest releases (#197)
6a90008 test: add integration tests for cycle draining (#198)
a2b993e docs: Add link to ICP Ninja in `basic_solana` README (#211)
24bb0cb build: Add CI pipeline job to check Ninja `Cargo.toml` (#159)
3955b8d fix: typo in Rust documentation of `ConsensusStrategy` (#205)
c3d5468 chore: upgrade SOL RPC canister to v1.1.0 (#206)
27a6a07 chore: Update README.md (#204)
0e0fd55 feat: deploy `basic_solana` on ICP Ninja (#152)
 ```

## Upgrade args

```
git fetch
git checkout 46f5c95e6d0200675d10e905fd20fbf8d4cf8968
didc encode -d canister/sol_rpc_canister.did -t '(InstallArgs)' '(record {})' | xxd -r -p | sha256sum
```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout 46f5c95e6d0200675d10e905fd20fbf8d4cf8968
"./scripts/docker-build"
sha256sum ./wasms/sol_rpc_canister.wasm.gz
```