#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_repo="${ADVERSARY_RAFT_LOG_GIT:-https://github.com/drmingdrmer/raft-log.git}"
results="${ADVERSARY_RESULTS_DIR:-${repo_root}/results/current/reader-boundary}"
work="$(mktemp -d /tmp/adversary-reader.XXXXXX)"
mkdir -p "${results}"
git clone -q "${source_repo}" "${work}"
for variant in parent fixed; do
  revision=707a9a2463ab215175ef9fbc8d86ea57c2aeac72
  if [[ "${variant}" == parent ]]; then revision="${revision}^"; fi
  # Each variant has a separate checkout; no restoration can erase experiment edits.
  checkout="${work}-${variant}"
  git -C "${work}" worktree add --quiet --detach "${checkout}" "${revision}"
  python3 "${repo_root}/scripts/prepare-reader-boundary.py" "${checkout}" \
    "${repo_root}/experiments/runtime-comparison/reader_boundary.rs" "${variant}"
  git -C "${checkout}" diff > "${results}/${variant}.patch"
  git -C "${checkout}" rev-parse HEAD > "${results}/${variant}-revision.txt"
  for level in 0 1 2; do
    expected=0
    if [[ "${variant}" == parent && "${level}" == 2 ]]; then expected=1; fi
    (cd "${checkout}" && READER_LEVEL="${level}" EXPECT_RACE="${expected}" \
      CARGO_TARGET_DIR=/tmp/adversary-reader-target CARGO_PROFILE_DEV_DEBUG=0 \
      cargo +nightly-2025-12-11 test -j 2 --lib adversary_reader_boundary -- --nocapture) \
      > "${results}/${variant}-level-${level}.log" 2>&1
  done
  if [[ "${variant}" == parent ]]; then
    python3 "${repo_root}/scripts/replay-reader-boundary.py" "${checkout}" "${results}"
  fi
done
