#!/usr/bin/env bash
set -euo pipefail

TARGET="wasm32-unknown-unknown"

# Optional manifest path
MANIFEST_ARG=""
if [ $# -ge 1 ]; then
  MANIFEST_ARG="--manifest-path $1"
fi

# Build based on the platform
if [ "$(uname)" == "Darwin" ]; then
  LLVM_PATH=$(brew --prefix llvm)
  AR="${LLVM_PATH}/bin/llvm-ar" CC="${LLVM_PATH}/bin/clang" cargo build --target "$TARGET" --release $MANIFEST_ARG
else
  cargo build --target "$TARGET" --release $MANIFEST_ARG
fi