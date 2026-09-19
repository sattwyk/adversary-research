# Continuation probes — 18 September 2026

These are executable mechanism checks supporting the accompanying Adversary continuation review. They are not a deterministic-testing platform, performance benchmark, filesystem model, or claim of finding a new redb bug.

## What was executed

| Probe | Actual implementation | Result |
|---|---|---|
| Component continuation | Wasmtime 48.0.2; two component instances in separate Stores; synchronous canonical imports implemented by asynchronous host functions | Nested guest frames survive suspension; host chooses interleaving, short write results, effects, persistence and completion. Four schedules each produce 101 identical event traces. |
| Real storage engine | Unmodified crates.io redb 4.1.0 compiled to a **core Wasm module**, using a custom StorageBackend | Commit and read succeed. Recovery succeeds after power loss during each of four commit writes, with and without persistence of the effected prefix. Nine crash scenarios each produce ten identical event traces, including sync taking effect before completion is returned. |
| Native continuation | corosensei 0.3.4, native Rust nested calls and std::io::Write::write_all | Two suspended continuations, short writes and a modeled sync barrier work in 100 repetitions. Dropping a suspended native coroutine runs application destructors, unlike the Wasm kill path. |

The redb guest uses `panic=abort`; an unexpected recovery error fails the run. A missing table represents the pre-transaction state. The normal run returns 42; the crash run may recover either the pre-transaction state or 42, depending on the operation. The tested sync-effect-before-completion case must recover 42.

The component probe uses a small WAT component, not production Rust. The redb probe uses real production Rust, but it has **not been componentized** and does not use WIT. These are complementary tests of the same Wasmtime suspension mechanism, not a completed Component Model SDK.

## Reproduce

Prerequisites: a Rust toolchain capable of building these locked dependencies, a supported native build environment for Wasmtime/Cranelift, and the `wasm32-unknown-unknown` Rust target. The original run used rustc 1.97.1 on x86_64 Windows. Cargo.lock files pin dependencies. No Wasmtime CLI is required.

From this directory:

```powershell
rustup target add wasm32-unknown-unknown
cargo build --manifest-path redb-guest/Cargo.toml --release --target wasm32-unknown-unknown --locked
cargo run --manifest-path host/Cargo.toml --locked --bin continuation-probe
cargo run --manifest-path host/Cargo.toml --locked --bin redb_probe -- redb-guest/target/wasm32-unknown-unknown/release/redb_guest.wasm
cargo run --manifest-path host/Cargo.toml --locked --bin native_fiber_probe
```

The same relative paths and cargo commands work in other shells on supported hosts. Compilation downloads dependencies unless they are cached. A first Wasmtime build can be substantial.

Recorded stdout is in `component-results.txt`, `redb-results.txt`, and `native-fiber-results.txt`. `integrity.txt` records the toolchain, verifies the redb registry archive against Cargo.lock, and compares all 71 packaged files with the compiled registry sources. These checks verify that the tested crate was not patched; they do not establish equivalence to a native build.

## Implementation and scope

- The single-threaded host explicitly polls futures; no Tokio executor drives the probes. Pending operations live outside Stores. Wakeups do not choose execution order.
- The component's live local canary survives two short writes and a subsequent sync. Node A can remain suspended while node B progresses.
- redb uses five storage operations: length, read, write, resize, and sync. Two extra imports mark workload phase and guest cleanup for assertions. No upstream redb source changes or async conversion were made.
- redb's write backend returns all-or-error. The partial-write tests kill the instance while a host operation is pending; they do not return a short success to that backend. Short successful writes are tested separately in the component and native probes.
- The storage model has volatile and durable byte vectors. Successful sync persists all current bytes. It is intentionally small and **not qualified against ext4, XFS, Windows, io_uring, or a device**. The partial-write choices are test scenarios, not a specification of hardware atomicity.
- Wasm kill means dropping the outstanding invocation and its Store, then constructing a fresh instance for recovery. The guest Drop marker is observed on normal completion and is absent on this suspended-call kill. Wasmtime still performs host-side cleanup.
- The component probe retains an issued operation after abandoning its waiter. The redb crash cases combine guest termination with power loss and discard outstanding model operations.
- The native probe's `flush` is a modeled sync barrier, not a native filesystem fsync. Its destructor result is a warning against treating ordinary coroutine cancellation as abrupt process death.
- Replay comparisons cover these selected event traces and asserted states within one build/host. They do not prove universal determinism, cross-version replay, bitwise cross-platform equivalence, or arbitrary schedule exploration.
- No throughput, latency, memory-overhead, distributed-service integration, historical-bug rediscovery, weak-memory, native-thread, concurrent-reader, or native-filesystem conformance result is claimed.

The pre-research architectural rejection criteria are preserved in `decision-criteria.md`.
