#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_repo="${ADVERSARY_RAFT_LOG_GIT:-https://github.com/drmingdrmer/raft-log.git}"
result_dir="${ADVERSARY_RESULTS_DIR:-${repo_root}/results/current/historical-raft-log}"
target_dir="${CARGO_TARGET_DIR:-/tmp/adversary-historical-raft-log-target}"
work_dir="$(mktemp -d /tmp/adversary-history-raft-log.XXXXXX)"

mkdir -p "${result_dir}"
git clone --quiet "${source_repo}" "${work_dir}"

run_state_case() {
  local revision="$1"
  local scenario="$2"
  local expected="$3"
  git -C "${work_dir}" switch --quiet --detach "${revision}"
  cp "${repo_root}/experiments/topology/examples/historical_state_safety.rs" \
    "${work_dir}/examples/historical_state_safety.rs"
  cd "${work_dir}"
  git rev-parse HEAD >> "${result_dir}/state-revisions.txt"
  HISTORICAL_SCENARIO="${scenario}" \
  EXPECT_REJECT="${expected}" \
  CARGO_TARGET_DIR="${target_dir}" \
  cargo +nightly-2025-12-11 run -j "${CARGO_BUILD_JOBS:-2}" \
    --example historical_state_safety \
    2>> "${result_dir}/build.log" | tee -a "${result_dir}/state.jsonl"
}

run_callback_case() {
  local revision="$1"
  local expected="$2"
  git -C "${work_dir}" switch --quiet --detach "${revision}"
  cp "${repo_root}/experiments/topology/examples/historical_callback.rs" \
    "${work_dir}/examples/historical_callback.rs"
  cd "${work_dir}"
  EXPECT_CALLBACK="${expected}" \
  CARGO_TARGET_DIR="${target_dir}" \
  cargo +nightly-2025-12-11 run -j "${CARGO_BUILD_JOBS:-2}" \
    --example historical_callback \
    2>> "${result_dir}/build.log" | tee -a "${result_dir}/callback.jsonl"
}

run_state_case ec79536^ commit-beyond-last 0
run_state_case ec79536 commit-beyond-last 1
run_state_case ec79536^ truncate-committed 0
run_state_case ec79536 truncate-committed 1
run_state_case c3370c6^ purge-conflicting-id 0
run_state_case c3370c6 purge-conflicting-id 1
# The callback fix predates the public flush(sync, callback) API. The retained
# historical_callback.rs is an unsuccessful integration attempt, not a working
# reproducer for those commits. Do not count it in this runnable corpus.
