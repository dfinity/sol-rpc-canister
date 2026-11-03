# Proposal to upgrade the SOL RPC canister

Repository: `https://github.com/dfinity/sol-rpc-canister.git`

Git hash: `7cc1089fa1160f12c18ac679720d35fb880ec119`

New compressed Wasm hash: `e63ca61947a7d5438180e656cdafe76a1b7bbca3c6ca1a57e2ff1cf4b32ccc71`

Upgrade args hash: `e04af557f2ed69b3396cb58bf6623e7fb645a275385f68e90cd880169a328b41`

Target canister: `tghme-zyaaa-aaaar-qarca-cai`

Previous SOL RPC proposal: https://dashboard.internetcomputer.org/proposal/139986

---

## Motivation

Upgrade the SOL RPC canister to the latest version [v1.4.0](https://github.com/dfinity/sol-rpc-canister/releases/tag/sol_rpc_canister-v1.4.0),
which includes in particular the following changes:
* Add support for the Solana `getMultipleAccounts` RPC method, to fetch up to 100 accounts in a single call.
* Upgrade various dependencies and bump Rust to 1.97.1.

See the GitHub release [v1.4.0](https://github.com/dfinity/sol-rpc-canister/releases/tag/sol_rpc_canister-v1.4.0) for more details.

## Release Notes

```
git log --format='%C(auto) %h %s' 9402b1ebb4fb14bab05b5bf5498894b9aea84198..7cc1089fa1160f12c18ac679720d35fb880ec119 --
7cc1089 chore: release (#402)
766ab67 feat(canister): add support for getMultipleAccounts (#394)
abb8ae6 build(deps): batch Cargo bumps (#393)
d9783f7 ci: de-flake end-to-end tests (#395)
931db0a build(deps): bump solana-nonce from 3.2.0 to 3.2.1 (#382)
753ff27 build(toolchain): bump Rust to 1.97.1 (#385)
0b8e813 build(deps): bump candid from 0.10.32 to 0.10.34 (#378)
0dbda99 build(deps): bump quinn-proto from 0.11.14 to 0.11.16 (#380)
fa6fd57 build(deps): consolidate Dependabot bumps that need no code changes (#379)
2955d23 build(deps): bump tokio from 1.52.3 to 1.53.1 (#372)
39222bb build(deps): bump thiserror from 2.0.18 to 2.0.19 (#371)
314bd93 build(deps): bump serde from 1.0.228 to 1.0.229 (#370)
65964ac chore(examples): migrate basic_solana from dfx to icp-cli (#375)
db84ab2 build(deps): consolidate Dependabot bumps that need no code changes (#367)
2ab5f25 build(deps): batch Cargo bumps (round 2) (#356)
300688b build(deps): bump http from 1.4.0 to 1.4.1 (#346)
14fe14c build(deps): batch Cargo bumps (ic-agent, candid, candid_parser, serde_json) (#344)
b0eb1aa build(deps): bump solana-clock from 3.0.1 to 3.1.0 (#339)
1acedad build(deps): bump tower-http from 0.6.10 to 0.6.11 (#342)
82f3027 build(deps): bump rand from 0.8.5 to 0.8.6 in /examples/basic_solana (#338)
7014125 build(deps): bump minicbor from 2.2.1 to 2.2.2 (#337)
5f4671b test: de-flake `should_get_recent_prioritization_fees` (#335)
b18c01d chore(deps): bump pocket-ic to v13 with companion runtime crates (#333)
7865d06 build(deps): bump rustls-webpki and quinn-proto in basic_solana example (#332)
f6a969e chore(deps): bump ic-agent to v0.47 with companion runtime crate (#334)
aedd6ff chore(deps): bulk cargo update consolidating Dependabot PRs (#331)
4fb5fec ci: pin ic-wasm to 0.9.10 (#320)
2229f19 ci: add checks-pass aggregator job (#322)
8d33f78 build(deps): revert all post-93a385b dependency bumps (#321)
9a50819 build(deps): bump pocket-ic from 12.0.0 to 13.0.0 (#311)
36c635f build(deps): bump solana-message from 3.1.0 to 4.1.1 (#312)
e943772 build(deps): bump ic-agent from 0.46.2 to 0.47.2 (#316)
c46068b build(deps): bump bincode from 1.3.3 to 3.0.0 (#314)
02d0abe build(deps): bump ic-management-canister-types from 0.5.0 to 0.7.1 (#315)
36030ac build(deps): bump solana-reward-info from 3.0.0 to 6.2.0 (#317)
93a385b build(deps): bump solana-client from 3.1.12 to 3.1.14 (#318)
af1c7fe chore: pin GitHub Actions to commit SHAs (#307)
00e2f22 build(deps): update dependencies (#309)
b0a1e33 build(deps): bump rustls-webpki from 0.103.10 to 0.103.12 (#308)
1f1e66f build(deps): bump keccak from 0.1.5 to 0.1.6 in /examples/basic_solana (#298)
4381752 build(deps): bump rustls-webpki from 0.103.9 to 0.103.10 (#302)
d4020d0 chore: release (#305)
c1a8aac build(deps): update dependencies (#304)
7eed6a2 refactor!: rename `estimate_recent_blockhash` to `get_recent_block` and return full block (#301)
98346cf refactor!: use `try_send` in `EstimateBlockhashRequestBuilder` (#300)
af65693 build(deps): bump quinn-proto from 0.11.13 to 0.11.14 (#299)
7c2d539 fix: bump `sol_rpc_canister` crate to v1.3.1 (#290)
94261bb ci: configure trusted publishing (#289)
acfb340 chore: release (#288)
37b4455 build(deps)!: update `ic-canister-runtime` to v0.2.0 (#287)
2180532 build: update dependencies (#279)
ad550d3 build(deps): bump time from 0.3.44 to 0.3.47 in /examples/basic_solana (#268)
e754b08 build(deps): upgrade dependencies (#266)
00efa70 chore: dfinity/defi-team -> dfinity/defi (#264)
5ec3382 chore: proposal to update to v1.3.0 (#263)
 ```

## Upgrade args

```
git fetch
git checkout 7cc1089fa1160f12c18ac679720d35fb880ec119
didc encode -d canister/sol_rpc_canister.did -t '(InstallArgs)' '(record {})' | xxd -r -p | sha256sum
```

## Wasm Verification

Verify that the hash of the gzipped WASM matches the proposed hash.

```
git fetch
git checkout 7cc1089fa1160f12c18ac679720d35fb880ec119
"./scripts/docker-build"
sha256sum ./wasms/sol_rpc_canister.wasm.gz
```