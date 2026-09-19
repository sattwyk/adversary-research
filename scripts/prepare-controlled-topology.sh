#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
destination="${ADVERSARY_TOPOLOGY_DIR:-${repo_root}/.work/controlled-topology}"

ADVERSARY_TOPOLOGY_DIR="${destination}" \
  bash "${repo_root}/scripts/prepare-topology.sh"

git -C "${destination}/chunked-wal" apply \
  "${repo_root}/experiments/topology/patches/chunked-wal-adversary.patch"
git -C "${destination}/databend-meta" apply \
  "${repo_root}/experiments/topology/patches/databend-controlled-probe.patch"

cp "${repo_root}/experiments/topology/examples/controlled_topology_probe.rs" \
  "${destination}/databend-meta/crates/server/raft-log/examples/controlled_topology_probe.rs"

printf '%s\n' "Prepared controlled topology sources in ${destination}"
