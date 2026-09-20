#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
topology_root="${ADVERSARY_TOPOLOGY_DIR:-/tmp/adversary-runtime-comparison}"
backend="${ADVERSARY_BACKEND:-native}"
result_dir="${ADVERSARY_RESULTS_DIR:-${repo_root}/results/current/runtime-${backend}}"
examples="${topology_root}/databend-meta/crates/server/raft-log/examples"
mkdir -p "${examples}/runtime_shared" "${result_dir}" "${topology_root}/data"
cp "${repo_root}/experiments/runtime-comparison/shared.rs" "${examples}/runtime_shared/mod.rs"
cp "${repo_root}/experiments/runtime-comparison/sync_failure.rs" "${examples}/runtime_shared/sync_failure.rs"
cp "${repo_root}/experiments/runtime-comparison/runtime_probe.rs" "${examples}/runtime_probe.rs"
features=()
case "${backend}" in
  native) ;;
  turmoil) features=(--features adversary-turmoil) ;;
  commonware) features=(--features adversary-commonware) ;;
  commonware-external) features=(--features adversary-commonware-external) ;;
  madsim) features=(--features adversary-madsim) ;;
  *) printf 'Unsupported backend: %s\n' "${backend}" >&2; exit 2 ;;
esac
cd "${topology_root}/databend-meta"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${XDG_CACHE_HOME:-$HOME/.cache}/adversary-runtime/target}"
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0
export RUSTFLAGS="${RUSTFLAGS:--C linker=cc}"
if [[ "${backend}" == madsim ]]; then
  export RUSTFLAGS="${RUSTFLAGS} --cfg madsim"
fi
export ADVERSARY_TEST_DIR="${topology_root}/data"
export ADVERSARY_TRIALS="${ADVERSARY_TRIALS:-10}"
export ADVERSARY_SCENARIO="${ADVERSARY_SCENARIO:-topology}"
export ADVERSARY_TRACE_DIR="${result_dir}/worker-traces"
date -u +%FT%TZ > "${result_dir}/started.txt"
printf '%s\n' "backend=${backend}" "toolchain=${ADVERSARY_TOOLCHAIN:-nightly-2025-12-11}" \
  "rustflags=${RUSTFLAGS}" "trials=${ADVERSARY_TRIALS}" "scenario=${ADVERSARY_SCENARIO}" > "${result_dir}/invocation.txt"
printf '%s\n' "allow_system_thread=${ADVERSARY_ALLOW_SYSTEM_THREAD:-0}" >> "${result_dir}/invocation.txt"
printf '%s\n' "serial_client=${ADVERSARY_SERIAL_CLIENT:-0}" "batch_items=${ADVERSARY_BATCH_ITEMS:-4}" \
  "process_timeout=${ADVERSARY_PROCESS_TIMEOUT:-60}" >> "${result_dir}/invocation.txt"
mkdir -p "${result_dir}/source"
cp "${repo_root}/experiments/runtime-comparison/"{shared,runtime_probe,sync_failure}.rs "${result_dir}/source/"
rustc "+${ADVERSARY_TOOLCHAIN:-nightly-2025-12-11}" -Vv > "${result_dir}/rustc.txt"
sha256sum "${repo_root}/experiments/runtime-comparison/shared.rs" \
  "${repo_root}/experiments/runtime-comparison/runtime_probe.rs" > "${result_dir}/source-sha256.txt"
sha256sum "${repo_root}/experiments/runtime-comparison/sync_failure.rs" >> "${result_dir}/source-sha256.txt"
set +e
cargo "+${ADVERSARY_TOOLCHAIN:-nightly-2025-12-11}" \
  --config "patch.crates-io.chunked-wal.path='${topology_root}/chunked-wal'" \
  build -j 2 -p databend-meta-raft-log --example runtime_probe "${features[@]}" \
  > "${result_dir}/build.log" 2>&1
build_outcome=$?
set -e
printf '%s\n' "${build_outcome}" > "${result_dir}/build-exit-code.txt"
cp Cargo.lock "${result_dir}/Cargo.lock"
if [[ "${build_outcome}" != 0 ]]; then
  date -u +%FT%TZ > "${result_dir}/finished.txt"
  exit "${build_outcome}"
fi
# External timeout covers native blocking calls that an async timeout cannot.
set +e
python3 "${repo_root}/scripts/measure-process.py" "${result_dir}" \
  "${CARGO_TARGET_DIR}/debug/examples/runtime_probe"
outcome=$?
set -e
printf '%s\n' "${outcome}" > "${result_dir}/exit-code.txt"
date -u +%FT%TZ > "${result_dir}/finished.txt"
exit "${outcome}"
