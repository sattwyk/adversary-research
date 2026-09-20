#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
export ADVERSARY_TOPOLOGY_DIR="${ADVERSARY_TOPOLOGY_DIR:-/tmp/adversary-runtime-comparison}"
bash "${repo_root}/scripts/prepare-controlled-topology.sh"
git -C "${ADVERSARY_TOPOLOGY_DIR}/databend-meta" apply \
  "${repo_root}/experiments/runtime-comparison/turmoil-manifest.patch"
git -C "${ADVERSARY_TOPOLOGY_DIR}/databend-meta" apply \
  "${repo_root}/experiments/runtime-comparison/additional-runtimes.patch"
