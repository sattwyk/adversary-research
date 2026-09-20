#!/usr/bin/env bash
# Sequential: each Cargo feature configuration writes the same example path.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
matrix_results="${ADVERSARY_MATRIX_RESULTS:-${repo_root}/results/current/runtime-matrix}"
for backend in native turmoil commonware-external madsim; do
  toolchain=nightly-2025-12-11
  if [[ "${backend}" == commonware-external ]]; then toolchain=nightly; fi
  for scenario in topology sync-failure; do
    ADVERSARY_BACKEND="${backend}" ADVERSARY_TOOLCHAIN="${toolchain}" \
      ADVERSARY_ALLOW_SYSTEM_THREAD=1 ADVERSARY_SCENARIO="${scenario}" \
      ADVERSARY_RESULTS_DIR="${matrix_results}/${backend}-${scenario}" \
      bash "${repo_root}/scripts/run-runtime-comparison.sh"
  done
done
python3 "${repo_root}/scripts/verify-runtime-matrix.py" "${matrix_results}"
