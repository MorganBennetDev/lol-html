#!/bin/sh

set -e

export CARGO_TARGET_DIR=$PWD/target

echo "===  Running library tests... ==="
cargo test --features=integration_test "$@"

echo "=== Building fuzzing test case code to ensure that it uses current API... ==="
(cd ./fuzz/test_case && cargo check)
