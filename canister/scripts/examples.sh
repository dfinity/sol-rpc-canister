#!/bin/bash

set -e -x

IDENTITY=${1:-ci}
NETWORK="ic"
FLAGS="--network=$NETWORK --identity=$IDENTITY"
${2:+FLAGS+=" --proxy=$2"}

# List supported JSON-RPC providers
icp canister call $FLAGS sol_rpc getProviders || exit 1

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
CYCLES=$(icp canister call $FLAGS sol_rpc getSlotCyclesCost "$GET_SLOT_PARAMS" | idl2json | jq '.Ok' --raw-output || exit 1)
GET_SLOT_OUTPUT=$(icp canister call $FLAGS --cycles "$CYCLES" sol_rpc getSlot "$GET_SLOT_PARAMS" | idl2json || exit 1)
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
CYCLES=$(icp canister call $FLAGS sol_rpc getRecentPrioritizationFeesCyclesCost "$GET_RECENT_PRIORITIZATION_FEES_PARAMS" | idl2json | jq '.Ok' --raw-output || exit 1)
GET_RECENT_PRIORITIZATION_FEES_OUTPUT=$(icp canister call $FLAGS --cycles "$CYCLES" sol_rpc getRecentPrioritizationFees "$GET_RECENT_PRIORITIZATION_FEES_PARAMS" | idl2json || exit 1)
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
    rewards = opt false;
  },
)"
CYCLES=$(icp canister call $FLAGS sol_rpc getBlockCyclesCost "$GET_BLOCK_PARAMS" | idl2json | jq '.Ok' --raw-output || exit 1)
GET_BLOCK_OUTPUT=$(icp canister call $FLAGS --cycles "$CYCLES" sol_rpc getBlock "$GET_BLOCK_PARAMS" | idl2json || exit 1)
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
CYCLES=$(icp canister call $FLAGS sol_rpc getSignatureStatusesCyclesCost "$GET_SIGNATURE_STATUSES_PARAMS" | idl2json | jq '.Ok' --raw-output || exit 1)
icp canister call $FLAGS --cycles "$CYCLES" sol_rpc getSignatureStatuses "$GET_SIGNATURE_STATUSES_PARAMS" || exit 1

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
CYCLES=$(icp canister call $FLAGS sol_rpc getTransactionCyclesCost "$GET_TRANSACTION_PARAMS" | idl2json | jq '.Ok' --raw-output || exit 1)
icp canister call $FLAGS --cycles "$CYCLES" sol_rpc getTransaction "$GET_TRANSACTION_PARAMS" || exit 1

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
CYCLES=$(icp canister call $FLAGS sol_rpc getAccountInfoCyclesCost "$GET_ACCOUNT_INFO_PARAMS" | idl2json | jq '.Ok' --raw-output || exit 1)
icp canister call $FLAGS --cycles "$CYCLES" sol_rpc getAccountInfo "$GET_ACCOUNT_INFO_PARAMS" || exit 1

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
CYCLES=$(icp canister call $FLAGS sol_rpc getBalanceCyclesCost "$GET_BALANCE_PARAMS" | idl2json | jq '.Ok' --raw-output || exit 1)
icp canister call $FLAGS --cycles "$CYCLES" sol_rpc getBalance "$GET_BALANCE_PARAMS" || exit 1

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
CYCLES=$(icp canister call $FLAGS sol_rpc getTokenAccountBalanceCyclesCost "$GET_TOKEN_ACCOUNT_BALANCE_PARAMS" | idl2json | jq '.Ok' --raw-output || exit 1)
icp canister call $FLAGS --cycles "$CYCLES" sol_rpc getTokenAccountBalance "$GET_TOKEN_ACCOUNT_BALANCE_PARAMS" || exit 1

# Get the last 10 USDC mint account transactions on Mainnet with a 2-out-of-3 strategy starting the search backwards
# from one of the transactions extracted from a block earlier.
GET_SIGNATURES_FOR_ADDRESS_PARAMS="(
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
    limit = opt (10 : nat32);
    before = opt \"${FIRST_SIGNATURE}\";
    until = null;
  },
)"
CYCLES=$(icp canister call $FLAGS sol_rpc getSignaturesForAddressCyclesCost "$GET_SIGNATURES_FOR_ADDRESS_PARAMS" | idl2json | jq '.Ok' --raw-output || exit 1)
icp canister call $FLAGS --cycles "$CYCLES" sol_rpc getSignaturesForAddress "$GET_SIGNATURES_FOR_ADDRESS_PARAMS" || exit 1