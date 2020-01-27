#!/usr/bin/env bash
set -ex
DESTINATION_FILE="$1"
ROOT="$3/../"

docker run --rm \
    -v "$ROOT":/code \
    -v "${HOME}/.cargo/registry":/root/.cargo/registry \
    -v "${HOME}/.cargo/git":/root/.cargo/git \
    softprops/lambda-rust

cp "$ROOT/target/lambda/release/elb-logs-to-cloudwatch.zip" "$DESTINATION_FILE"

