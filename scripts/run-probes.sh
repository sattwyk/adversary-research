#!/usr/bin/env bash
# Rebuild and run the completed continuation experiments; never infer success
# from old log files. Any failed build/assertion makes this script fail.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$root"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/.work/target}"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}"
results="${ADVERSARY_RESULTS_DIR:-$root/results/current}"
mkdir -p "$results"
host=experiments/continuations/host/Cargo.toml
guest=experiments/continuations/redb-guest/Cargo.toml
cargo fmt --manifest-path "$host" -- --check
cargo fmt --manifest-path "$guest" -- --check
cargo build --manifest-path "$guest" --release --target wasm32-unknown-unknown --locked
cargo clippy --manifest-path "$host" --all-targets --locked -- -D warnings
cargo run --manifest-path "$host" --locked --bin continuation-probe | tee "$results/component-results.txt"
cargo run --manifest-path "$host" --locked --bin redb_probe -- "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/redb_guest.wasm" | tee "$results/redb-results.txt"
cargo run --manifest-path "$host" --locked --bin native_fiber_probe | tee "$results/native-fiber-results.txt"
{
  date -u +'%Y-%m-%dT%H:%M:%SZ'
  rustc -Vv
  cargo -V
  uname -sm
  sha256sum "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/redb_guest.wasm"
} > "$results/environment.txt"
printf 'All continuation probes passed. Results: %s\n' "$results"
