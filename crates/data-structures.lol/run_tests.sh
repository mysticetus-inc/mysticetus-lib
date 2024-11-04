#! /usr/bin/env bash
CHECKPOINT_FILE="loom_tests.json"

LOOM_LOG=trace \
    LOOM_LOCATION=1 \
    LOOM_CHECKPOINT_INTERVAL=10 \
    LOOM_CHECKPOINT_FILE=$CHECKPOINT_FILE \
    RUSTFLAGS="--cfg loom" \
    cargo test --release loom_test_atomic_cell -- --nocapture

# delete the checkpoint file if all tests pass (i.e cargo test returns a success error code)
fi