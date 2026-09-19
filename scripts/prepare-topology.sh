#!/usr/bin/env bash
# Restore pinned upstream sources locally, then overlay only our experiment.
# Refuse to overwrite an existing checkout: local edits should remain reviewable.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
destination="${ADVERSARY_TOPOLOGY_DIR:-$root/.work/topology}"
if [[ -e "$destination" ]]; then
  printf 'Destination already exists; inspect/reuse it explicitly: %s\n' "$destination" >&2
  exit 1
fi
mkdir -p "$destination"
while IFS=$'\t' read -r component revision origin; do
  [[ "$component" == component ]] && continue
  tar -xzf "$root/archive/reference-snapshots/$component-$revision.tar.gz" -C "$destination"
  patch="$root/archive/reference-snapshots/$component-working-tree.patch"
  if [[ -s "$patch" ]]; then
    (cd "$destination/$component" && git apply "$patch")
  fi
done < "$root/archive/reference-snapshots/topology-pins.tsv"
mkdir -p "$destination/databend-meta/crates/server/raft-log/examples"
cp "$root/experiments/topology/examples/topology_probe.rs" "$destination/databend-meta/crates/server/raft-log/examples/topology_probe.rs"
cp "$root/experiments/topology/Cargo.lock" "$destination/databend-meta/Cargo.lock"
printf 'Prepared pinned topology experiment: %s\n' "$destination"
