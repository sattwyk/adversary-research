#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
source_repo="${ADVERSARY_CHUNKED_WAL_GIT:-https://github.com/drmingdrmer/chunked-wal.git}"
result_dir="${ADVERSARY_RESULTS_DIR:-${repo_root}/results/current/historical-zero-tail}"
work_dir="$(mktemp -d /tmp/adversary-history-zero-tail.XXXXXX)"
mkdir -p "${result_dir}"
git clone --quiet "${source_repo}" "${work_dir}"
cd "${work_dir}"
for revision in 938cc41 f71b906; do
  git switch --quiet --detach "${revision}"
  git rev-parse HEAD > "${result_dir}/${revision}.revision"
  CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/adversary-historical-target-cw}" \
    cargo +nightly-2025-12-11 test -j 2 --test corruption --no-run \
    > "${result_dir}/${revision}.build.log" 2>&1
  set +e
  CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/adversary-historical-target-cw}" \
    cargo +nightly-2025-12-11 test -j 2 --test corruption \
    test_zero_filled_tail_chunk_is_removed -- --include-ignored --exact \
    > "${result_dir}/${revision}.test.log" 2>&1
  outcome=$?
  set -e
  printf '%s\n' "${outcome}" > "${result_dir}/${revision}.exit-code"
  if [[ "${revision}" == 938cc41 ]]; then
    test "${outcome}" -eq 101
  else
    test "${outcome}" -eq 0
  fi
done
printf '%s\n' 'Observed parent failure and fixed-revision pass for zero-filled tail recovery.'
