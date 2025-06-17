#!/bin/bash

set -e -x

NETWORK="ic"
IDENTITY="hsm"
WALLET=$(dfx identity get-wallet --network=$NETWORK --identity=$IDENTITY)
CANISTER="sol_rpc"
FLAGS="--network=$NETWORK --identity=$IDENTITY --wallet=$WALLET"

dfx canister call ${CANISTER} updateApiKeys "(vec {
  record { variant { AnkrMainnet }; opt \"${ANKR_API_KEY}\" };
  record { variant { AnkrDevnet }; opt \"${ANKR_API_KEY}\" };
})" ${FLAGS}

