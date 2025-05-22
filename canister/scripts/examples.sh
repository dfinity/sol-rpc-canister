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
GET_SLOT_OUTPUT=$(dfx canister call sol_rpc getSlot "$GET_SLOT_PARAMS" $FLAGS --output json --with-cycles "$CYCLES" || exit 1)
SLOT=$(jq --raw-output '.Consistent.Ok' <<< "$GET_SLOT_OUTPUT")


# Get the recent prioritization fees on Mainnet with a 2-out-of-3 strategy for USDC
GET_RECENT_PRIORITIZATION_FEES_PARAMS="(
  variant { Default = variant { Mainnet } },
  opt record {
    responseConsensus = opt variant {
      Threshold = record { min = 2 : nat8; total = opt (3 : nat8) }
    };
    responseSizeEstimate = null;
    maxSlotRoundingError = opt (20 : nat64);
    maxLength = opt (100 : nat8);
  },
  opt vec { \"EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v\" },
)"
CYCLES=$(dfx canister call sol_rpc getRecentPrioritizationFeesCyclesCost "$GET_RECENT_PRIORITIZATION_FEES_PARAMS" $FLAGS --output json | jq '.Ok' --raw-output || exit 1)
GET_RECENT_PRIORITIZATION_FEES_OUTPUT=$(dfx canister call sol_rpc getRecentPrioritizationFees "$GET_RECENT_PRIORITIZATION_FEES_PARAMS" $FLAGS --output json --with-cycles "$CYCLES" || exit 1)
GET_RECENT_PRIORITIZATION_FEES=$(jq --raw-output '.Consistent.Ok' <<< "$GET_RECENT_PRIORITIZATION_FEES_OUTPUT")

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
    transactionDetails = opt variant { signatures };
    maxSupportedTransactionVersion = opt (0 : nat8);
  },
)"
CYCLES=$(dfx canister call sol_rpc getBlockCyclesCost "$GET_BLOCK_PARAMS" $FLAGS --output json | jq '.Ok' --raw-output || exit 1)
GET_BLOCK_OUTPUT=$(dfx canister call sol_rpc getBlock "$GET_BLOCK_PARAMS" $FLAGS --output json --with-cycles "$CYCLES" || exit 1)
FIRST_SIGNATURE=$(jq --raw-output '.Consistent.Ok[0].signatures[0][0]' <<< "$GET_BLOCK_OUTPUT")
SECOND_SIGNATURE=$(jq --raw-output '.Consistent.Ok[0].signatures[0][1]' <<< "$GET_BLOCK_OUTPUT")

# Fetch the statuses of the first two transactions in the received block
GET_SIGNATURE_STATUSES_PARAMS="(
  variant { Default = variant { Mainnet } },
  opt record {
    responseConsensus = opt variant {
      Threshold = record { min = 2 : nat8; total = opt (3 : nat8) }
    };
    responseSizeEstimate = null;
  },
  record {
    signatures = vec { \"${FIRST_SIGNATURE}\"; \"${SECOND_SIGNATURE}\" };
    searchTransactionHistory = null;
  },
)"
CYCLES=$(dfx canister call sol_rpc getSignatureStatusesCyclesCost "$GET_SIGNATURE_STATUSES_PARAMS" $FLAGS --output json | jq '.Ok' --raw-output || exit 1)
dfx canister call sol_rpc getSignatureStatuses "$GET_SIGNATURE_STATUSES_PARAMS" $FLAGS --with-cycles "$CYCLES" || exit 1

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
    signature = \"${FIRST_SIGNATURE}\";
    commitment = opt variant { finalized };
    encoding = opt variant{ base64 };
    maxSupportedTransactionVersion = opt (0 : nat8);
  },
)"
CYCLES=$(dfx canister call sol_rpc getTransactionCyclesCost "$GET_TRANSACTION_PARAMS" $FLAGS --output json | jq '.Ok' --raw-output || exit 1)
dfx canister call sol_rpc getTransaction "$GET_TRANSACTION_PARAMS" $FLAGS --with-cycles "$CYCLES" || exit 1

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

# Get the USDC mint account balance on Mainnet with a 2-out-of-3 strategy
GET_BALANCE_PARAMS="(
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
    minContextSlot = null;
  },
)"
CYCLES=$(dfx canister call sol_rpc getBalanceCyclesCost "$GET_BALANCE_PARAMS" $FLAGS --output json | jq '.Ok' --raw-output || exit 1)
dfx canister call sol_rpc getBalance "$GET_BALANCE_PARAMS" $FLAGS --with-cycles "$CYCLES" || exit 1

# Get the USDC issuer (Circle) token account balance on Mainnet with a 2-out-of-3 strategy
GET_TOKEN_ACCOUNT_BALANCE_PARAMS="(
  variant { Default = variant { Mainnet } },
  opt record {
    responseConsensus = opt variant {
      Threshold = record { min = 2 : nat8; total = opt (3 : nat8) }
    };
    responseSizeEstimate = null;
  },
  record {
    pubkey = \"3emsAVdmGKERbHjmGfQ6oZ1e35dkf5iYcS6U4CPKFVaa\";
    commitment = null;
  },
)"
CYCLES=$(dfx canister call sol_rpc getTokenAccountBalanceCyclesCost "$GET_TOKEN_ACCOUNT_BALANCE_PARAMS" $FLAGS --output json | jq '.Ok' --raw-output || exit 1)
dfx canister call sol_rpc getTokenAccountBalance "$GET_TOKEN_ACCOUNT_BALANCE_PARAMS" $FLAGS --with-cycles "$CYCLES" || exit 1