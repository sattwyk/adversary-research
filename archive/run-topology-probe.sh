#!/usr/bin/env bash
set -euo pipefail
cd /mnt/c/Users/sattw/Documents/Codex/2026-09-18/you-are-evaluating-a-systems-architecture/work/topology-experiment/sources/databend-meta
export CARGO_TARGET_DIR=/tmp/adversary-topology-20260918/work/target
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export RUSTFLAGS='-C linker=cc'
export ADVERSARY_TEST_DIR=/tmp/adversary-topology-20260918/work/native-wal
mkdir -p "$ADVERSARY_TEST_DIR"
cargo +nightly-2025-12-11 run -j 4 -p databend-meta-raft-log --example topology_probe > ../../logs/native-topology-trials.jsonl 2> ../../logs/native-topology-build.log
