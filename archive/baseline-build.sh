#!/usr/bin/env bash
set -euo pipefail
cd /mnt/c/Users/sattw/Documents/Codex/2026-09-18/you-are-evaluating-a-systems-architecture/work/topology-experiment/sources/databend-meta
export CARGO_TARGET_DIR=/tmp/adversary-topology-20260918/work/target
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export CARGO_INCREMENTAL=0
export RUSTFLAGS='-C linker=cc'
cargo +nightly-2025-12-11 test -j 4 -p databend-meta-raft-log --no-run 2>&1 | tee ../../logs/baseline-build-cc.log
