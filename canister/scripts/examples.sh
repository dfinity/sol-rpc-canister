#!/bin/bash

set -e -x

IDENTITY=${1:-ci}
NETWORK="ic"
WALLET=$(dfx identity get-wallet --network="$NETWORK" --identity="$IDENTITY")
FLAGS="--network=$NETWORK --identity=$IDENTITY --wallet=$WALLET"

# List supported JSON-RPC providers
dfx canister call sol_rpc getProviders $FLAGS || exit 1

# Get the last finalized slot on Mainnet with a 2-out-of-3 strategy
# TODO XC-321: get cycle cost by query method
CYCLES="2B"
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
dfx canister call sol_rpc getSlot "$GET_SLOT_PARAMS" $FLAGS --with-cycles "$CYCLES" || exit 1

# Get the System Program account info on Mainnet with a 2-out-of-3 strategy
# TODO XC-321: get cycle cost by query method
CYCLES="2B"
GET_ACCOUNT_INFO_PARAMS="(
  variant { Default = variant { Mainnet } },
  opt record {
    responseConsensus = opt variant {
      Threshold = record { min = 2 : nat8; total = opt (3 : nat8) }
    };
    responseSizeEstimate = null;
  },
  vec { 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0; 0 },
  opt record {
    commitment = null;
    encoding = variant{ base64 };
    dataSlice = null;
    minContextSlot = null;
  },
)"
dfx canister call sol_rpc getAccountInfo "$GET_ACCOUNT_INFO_PARAMS" $FLAGS --with-cycles "$CYCLES" || exit 1
