#!/usr/bin/env bash
# Native worker observation only. This is not deterministic integration or a
# power-loss test; the workload explicitly closes and reopens the real WAL.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
destination="${ADVERSARY_TOPOLOGY_DIR:-$root/.work/topology}"
if [[ ! -f "$destination/databend-meta/Cargo.toml" ]]; then
  printf 'Prepare sources with bash scripts/prepare-topology.sh first.\n' >&2
  exit 1
fi
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/.work/topology-target}"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0
export RUSTFLAGS="${RUSTFLAGS:--C linker=cc}"
export ADVERSARY_TEST_DIR="${ADVERSARY_TEST_DIR:-$destination/native-wal}"
results="${ADVERSARY_RESULTS_DIR:-$root/results/current/topology}"
mkdir -p "$ADVERSARY_TEST_DIR" "$results"
cd "$destination/databend-meta"
cargo +nightly-2025-12-11 run --locked -j "${CARGO_BUILD_JOBS:-2}" -p databend-meta-raft-log --example topology_probe 2> "$results/build.log" | tee "$results/trials.jsonl"
