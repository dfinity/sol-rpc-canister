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
SLOT=$(dfx canister call sol_rpc getSlot "$GET_SLOT_PARAMS" $FLAGS --with-cycles "$CYCLES" --output json | jq '.Consistent.Ok' --raw-output || exit 1 | tee /dev/tty)

# Fetch the latest finalized block
GET_BLOCK_PARAMS="(
  variant { Default = variant { Mainnet } },
  opt record {
    responseConsensus = opt variant {
      Threshold = record { min = 2 : nat8; total = opt (3 : nat8) }
    };
    responseSizeEstimate = null;
  },
  record {
    slot = ${SLOT};
    commitment = opt variant { finalized };
    maxSupportedTransactionVersion = null;
  },
)"
CYCLES=$(dfx canister call sol_rpc getBlockCyclesCost "$GET_BLOCK_PARAMS" $FLAGS --output json | jq '.Ok' --raw-output || exit 1)
SIGNATURE=$(dfx canister call sol_rpc getBlock "$GET_BLOCK_PARAMS" $FLAGS --with-cycles "$CYCLES" | jq '.Consistent.Ok.signatures[0]' --raw-output || exit 1 | tee /dev/tty)

# Fetch the first transaction in the retrieved block
GET_TRANSACTION_PARAMS="(
  variant { Default = variant { Mainnet } },
  opt record {
    responseConsensus = opt variant {
      Threshold = record { min = 2 : nat8; total = opt (3 : nat8) }
    };
    responseSizeEstimate = null;
  },
  record {
    signature = ${SIGNATURE};
    commitment = opt variant { finalized };
    encoding = opt variant{ base64 };
    maxSupportedTransactionVersion = null;
  },
)"
CYCLES=$(dfx canister call sol_rpc getTransactionCyclesCost "$GET_TRANSACTION_PARAMS" $FLAGS --output json | jq '.Ok' --raw-output || exit 1)
dfx canister call sol_rpc getTransaction "$GET_TRANSACTION_PARAMS" $FLAGS --with-cycles "$CYCLES" || exit 1

# TODO XC-339: Add end-to-end test for `sendTransaction` using `getSlot` and `getBlock`

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