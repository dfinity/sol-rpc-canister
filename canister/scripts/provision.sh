#!/bin/bash

set -e -x

# Pass API keys by environment variables
# Fail if the variable is not set
set -u # or set -o nounset
: "$ALCHEMY_MAINNET_API_KEY"
: "$ALCHEMY_DEVNET_API_KEY"
: "$ANKR_MAINNET_API_KEY"
: "$DRPC_DEVNET_API_KEY"
: "$DRPC_MAINNET_API_KEY"
: "$ANKR_DEVNET_API_KEY"
: "$HELIUS_MAINNET_API_KEY"
: "$HELIUS_DEVNET_API_KEY"

NETWORK="ic"
IDENTITY="ci"
WALLET=$(dfx identity get-wallet --network=$NETWORK --identity=$IDENTITY)
CANISTER="sol_rpc"
FLAGS="--network=$NETWORK --identity=$IDENTITY --wallet=$WALLET"

dfx canister call ${CANISTER} updateApiKeys "(vec {
  record { variant { AlchemyMainnet }; opt \"${ALCHEMY_MAINNET_API_KEY}\" };
  record { variant { AlchemyDevnet }; opt \"${ALCHEMY_DEVNET_API_KEY}\" };
  record { variant { AnkrMainnet }; opt \"${ANKR_MAINNET_API_KEY}\" };
  record { variant { AnkrDevnet }; opt \"${ANKR_DEVNET_API_KEY}\" };
  record { variant { DrpcMainnet }; opt \"${DRPC_MAINNET_API_KEY}\" };
  record { variant { DrpcDevnet }; opt \"${DRPC_DEVNET_API_KEY}\" };
  record { variant { HeliusMainnet }; opt \"${HELIUS_MAINNET_API_KEY}\" };
  record { variant { HeliusDevnet }; opt \"${HELIUS_DEVNET_API_KEY}\" };
})" ${FLAGS}

