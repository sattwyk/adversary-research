#!/usr/bin/env bash
# This is a negative test of the controller, not an alleged production bug.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
results="${ADVERSARY_ATTACK_RESULTS:-${repo_root}/results/current/checkpoint-gate}"
for quota in 4 1; do
  set +e
  ADVERSARY_BACKEND=native ADVERSARY_SCENARIO=topology ADVERSARY_TRIALS=1 \
    ADVERSARY_SERIAL_CLIENT=1 ADVERSARY_BATCH_ITEMS="${quota}" ADVERSARY_PROCESS_TIMEOUT=5 \
    ADVERSARY_RESULTS_DIR="${results}/quota-${quota}" \
    bash "${repo_root}/scripts/run-runtime-comparison.sh"
  outcome=$?
  set -e
  if [[ "${quota}" == 4 ]]; then test "${outcome}" == 124; else test "${outcome}" == 0; fi
done
