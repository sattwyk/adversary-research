# Continuation experiments

These binaries are assertion-driven experiments, not a library API. Read the module comments before changing their execution oracles.

## Code tour

| File | Responsibility |
|---|---|
| `host/fixtures/commit.wat` | A component with synchronous write/sync imports, a short-write loop and nested frames with a live canary |
| `host/src/main.rs` | A deterministic host driver manually polling two Store-owned invocations; storage effect and completion delivery are separate |
| `host/src/bin/redb_probe.rs` | Core-Wasm host running real redb commit/recovery against a volatile/durable byte-vector disk |
| `redb-guest/src/lib.rs` | Five-operation StorageBackend adapter; workload-phase and cleanup instrumentation; unmodified redb dependency |
| `host/src/bin/native_fiber_probe.rs` | Native stackful comparison using actual Rust `write_all`, plus an application destructor marker |

## Individual commands

From the repository root, choose one build directory and use it for both crates:

```bash
export CARGO_TARGET_DIR="$PWD/.work/target"
cargo build --manifest-path experiments/continuations/redb-guest/Cargo.toml --release --target wasm32-unknown-unknown --locked
cargo run --manifest-path experiments/continuations/host/Cargo.toml --locked --bin continuation-probe
cargo run --manifest-path experiments/continuations/host/Cargo.toml --locked --bin redb_probe -- "$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/redb_guest.wasm"
cargo run --manifest-path experiments/continuations/host/Cargo.toml --locked --bin native_fiber_probe
```

`bash scripts/run-probes.sh` also checks formatting/lints and captures dated environment information.

## What the assertions establish

**Component probe.** Four combinations of kill/no-kill and persist/don't-persist are each run 101 times. A local canary survives two short writes and sync. Another Store progresses while the first waits. Dropping a waiter does not remove its simulator-owned request. This is one manually specified family of schedules, not an explorer.

**redb probe.** A real transaction creates an item with value 42. The driver kills halfway through each of four commit writes, with/without persistence of that prefix, and reopens a fresh guest. A separate scenario kills after sync affects durable state but before its completion returns. Recovery permits the old state or the expected committed value; the sync-effect scenario requires 42. Only `TableDoesNotExist` counts as an absent table; unexpected recovery errors fail. Each of nine crash scenarios runs ten times.

The redb backend has all-or-error write semantics. Partial writes here are effects interrupted by a crash, not short successful returns from redb's backend API. The component/native probes separately test short successful `write_all` progress.

**Native fiber probe.** Two native coroutines retain their call stacks over short writes. The first completes; dropping the second runs its cleanup marker. Its `Write::flush` is a model barrier, not a real fsync. The test exposes the difference between cancellation cleanup and abrupt process termination.

## Deliberate limitations

- The real redb guest is core Wasm; only the small WAT fixture is a component. There is no finished WIT SDK.
- The disk model is not qualified against ext4, XFS, Windows, io_uring or hardware.
- No native concurrent readers, weak-memory cases, whole-runtime portability or throughput are tested.
- Trace equality is checked within a pinned execution environment. Cross-platform execution is recorded separately and is not a general determinism proof.
- The baseline redb run closes normally. The sync-effect-before-completion crash case separately demonstrates recovery without normal guest cleanup at that point.
- Read delivery uses this sequential model's current bytes; a general concurrent read-linearization contract is out of scope.
- Host-side cleanup still runs when a Wasmtime future is dropped. Guest resource/handle lifetimes need additional tests before generalization.

Historical source and stdout are preserved under `archive/` and `results/2026-09-18/`. Added comments and formatting change source-line counts, not the scope of the original evidence.
