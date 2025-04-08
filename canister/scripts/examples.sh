#!/bin/bash

set -e -x

IDENTITY=${1:-ci}
NETWORK="ic"
WALLET=$(dfx identity get-wallet --network="$NETWORK" --identity="$IDENTITY")
FLAGS="--network=$NETWORK --identity=$IDENTITY --wallet=$WALLET"

# List supported JSON-RPC providers
dfx canister call sol_rpc getProviders $FLAGS || exit 1

# Get the last finalized slot on Mainnet with a 2-out-of-3 strategy
GET_SLOT_PARAMS="(
  variant { Default = variant { Mainnet } },
  opt record {
    responseConsensus = opt variant {
      Threshold = record { min = 2 : nat8; total = opt (3 : nat8) }
    };
    responseSizeEstimate = null;
  },
  opt record { minContextSlot = null; commitment = opt variant { finalized } },
)"
CYCLES=$(dfx canister call sol_rpc getSlotCyclesCost "$GET_SLOT_PARAMS" $FLAGS --output json | jq '.Ok' --raw-output || exit 1)
dfx canister call sol_rpc getSlot "$GET_SLOT_PARAMS" $FLAGS --with-cycles "$CYCLES" || exit 1

# Get the USDC mint account info on Mainnet with a 2-out-of-3 strategy
GET_ACCOUNT_INFO_PARAMS="(
  variant { Default = variant { Mainnet } },
  opt record {
    responseConsensus = opt variant {
      Threshold = record { min = 2 : nat8; total = opt (3 : nat8) }
    };
    responseSizeEstimate = null;
  },
  record {
    pubkey = \"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v\";
    commitment = null;
    encoding = opt variant{ base64 };
    dataSlice = null;
    minContextSlot = null;
  },
)"
CYCLES=$(dfx canister call sol_rpc getAccountInfoCyclesCost "$GET_ACCOUNT_INFO_PARAMS" $FLAGS --output json | jq '.Ok' --raw-output || exit 1)
dfx canister call sol_rpc getAccountInfo "$GET_ACCOUNT_INFO_PARAMS" $FLAGS --with-cycles "$CYCLES" || exit 1
