#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
topology_root="${ADVERSARY_TOPOLOGY_DIR:-${repo_root}/.work/controlled-topology}"
target_dir="${CARGO_TARGET_DIR:-${repo_root}/.work/topology-target}"
result_dir="${ADVERSARY_RESULTS_DIR:-${repo_root}/results/current/controlled-topology}"
test_dir="${ADVERSARY_TEST_DIR:-${topology_root}/controlled-wal}"
example="${ADVERSARY_EXAMPLE:-controlled_topology_probe}"

mkdir -p "${result_dir}" "${test_dir}"

cp "${repo_root}/experiments/topology/examples/${example}.rs" \
  "${topology_root}/databend-meta/crates/server/raft-log/examples/${example}.rs"

# The prepared source has the experiment-only manifest and chunked-wal source
# overlaid by prepare-controlled-topology.sh. Cargo's patch keeps raft-log's
# dependency edge and version requirement intact while selecting that checkout.
cd "${topology_root}/databend-meta"
ADVERSARY_TEST_DIR="${test_dir}" \
ADVERSARY_TRIALS="${ADVERSARY_TRIALS:-10}" \
CARGO_TARGET_DIR="${target_dir}" \
CARGO_PROFILE_DEV_DEBUG=0 \
CARGO_INCREMENTAL=0 \
RUSTFLAGS="${RUSTFLAGS:--C linker=cc}" \
cargo +nightly-2025-12-11 \
  --config "patch.crates-io.chunked-wal.path='${topology_root}/chunked-wal'" \
  run -j "${CARGO_BUILD_JOBS:-2}" \
  -p databend-meta-raft-log \
  --example "${example}" \
  2> "${result_dir}/build.log" | tee "${result_dir}/trials.jsonl"
