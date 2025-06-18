#!/bin/bash

set -e -x

# Set a dummy value for the API keys
DUMMY_API_KEY="dummy"

dfx canister call sol_rpc updateApiKeys "(vec {
  record { variant { AlchemyMainnet }; opt \"${DUMMY_API_KEY}\" };
  record { variant { AlchemyDevnet }; opt \"${DUMMY_API_KEY}\" };
  record { variant { AnkrMainnet }; opt \"${DUMMY_API_KEY}\" };
  record { variant { AnkrDevnet }; opt \"${DUMMY_API_KEY}\" };
  record { variant { ChainstackMainnet }; opt \"${DUMMY_API_KEY}\" };
  record { variant { ChainstackDevnet }; opt \"${DUMMY_API_KEY}\" };
  record { variant { DrpcMainnet }; opt \"${DUMMY_API_KEY}\" };
  record { variant { DrpcDevnet }; opt \"${DUMMY_API_KEY}\" };
  record { variant { HeliusMainnet }; opt \"${DUMMY_API_KEY}\" };
  record { variant { HeliusDevnet }; opt \"${DUMMY_API_KEY}\" };
})"

