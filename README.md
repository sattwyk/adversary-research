# Adversary research

An adversarial investigation of deterministic testing **above the OS**, preserving production correctness code through controlled external effects and execution continuations.

**Status: research repository with executable mechanism probes. This is not a finished deterministic-testing platform.** The continuation experiments succeeded on their declared workloads. The full distributed execution-topology experiment remains unfinished.

## Start here

1. [Evidence and project status](docs/STATUS.md): what ran, what failed, and what remains untested.
2. [Full continuation review](docs/research/continuation-review.md): the main architecture analysis, implementation references, alternatives, and falsification protocols.
3. [Production topology](docs/research/production-topology.md): Databend Meta / OpenRaft / raft-log / chunked-wal workers, callbacks, queues, locks and durability.
4. [Experiment guide](experiments/continuations/README.md): code tour and reproducible continuation probes.
5. [Reviewer guide](docs/REVIEWING.md): invariants to inspect and claims to challenge.
6. [Research history](docs/HISTORY.md): how the conclusion changed, including the original RFC assessment.

The working conclusion is conditional **H2**: suspended Wasm imports can retain meaningful synchronous production code. The unresolved question is whether enough production concurrency and native behavior survive to justify the integration cost over existing runtimes or native execution.

## Evidence at a glance

| Experiment | Preserved implementation | Recorded result | Limits |
|---|---|---|---|
| Synchronous Wasmtime components | Nested WAT call frames, short-write loop, live local value | Four selected schedules × 101 identical traces in the original run | Small mechanism fixture; no guest threads or performance claim |
| redb 4.1.0 | Unmodified transaction, page and recovery engine | Nine crash scenarios × ten matching traces; recovery after partial writes and sync-before-completion | Core Wasm, one caller, abstract byte-vector storage; not native filesystem validation |
| Native corosensei fibers | Native Rust `Write::write_all` and nested locals | 100 repetitions; normal coroutine Drop executes application cleanup | Demonstrates why dropping a fiber is not abrupt process death |
| Databend Raft storage baseline | Pinned upstream package tests | Original logs: 11 unit tests + 1 recovery integration test passed | Package baseline, not a simulated cluster |
| Native topology probe | Real Raft log encoding, queueing, flush worker and recovery | Repaired example: 20/20 graceful recovery trials passed; 20 distinct batch sequences observed | Native observation only; original failed build and empty log are retained |

Results are dated evidence. Passing an old log is not a substitute for running the current code. No new production defect, execution-throughput advantage, qualified filesystem correspondence, or complete distributed integration is claimed.

## Run the completed probes

Use Linux/WSL with Rust **1.97.1**, a native C toolchain, and the Wasm target:

```bash
rustup toolchain install 1.97.1 --profile minimal --component rustfmt --component clippy --target wasm32-unknown-unknown
rustup override set 1.97.1
bash scripts/run-probes.sh
```

The script checks formatting, runs Clippy, compiles the unchanged redb dependency to Wasm, then executes all three host probes. It exits unsuccessfully on any failed build or assertion. New logs go to ignored `results/current/`.

Build products default to `.work/target/`. When working from a Windows-mounted checkout, keep large build products in the WSL filesystem:

```bash
CARGO_TARGET_DIR=/tmp/adversary-research-build bash scripts/run-probes.sh
```

Dependency versions are locked. Initial compilation of Wasmtime can take several minutes. See [individual commands and assertions](experiments/continuations/README.md). GitHub Actions runs the same continuation script; the larger, unfinished topology experiment is intentionally separate.

## Repository map

```text
docs/
  briefs/                 Original RFC and research objectives
  history/                Earlier answers, rejection criteria and completion audit
  research/               Full review and production-topology record
experiments/
  continuations/host/     Component, redb host and native-fiber probes
  continuations/redb-guest/ Thin adapter; redb itself is a locked dependency
  topology/              Native worker observation, lockfile and protocol
results/
  2026-09-18/             Original Windows probes and topology build evidence
  2026-09-19/             Repository cleanup/revalidation results
archive/
  reference-snapshots/    Pinned upstream source archives, SQLite sources and patches
  continuation-probes-original.zip  Original delivered code, before cleanup
scripts/                 Repeatable build/run and topology preparation
```

The initial Git commit preserves the original material. Subsequent commits make the code readable and runnable without rewriting the historical findings. Original source-line counts in reports refer to the pre-cleanup code.

## What was cleaned up

- Added module/function comments about ownership, suspension, buffer lifetimes and crash semantics.
- Extracted the component fixture into a readable `.wat` file.
- Replaced numeric crash sentinels with a named enum and improved helper names.
- Formatted Rust and added a single reproducible validation command and CI.
- Preserved historical failures and empty results rather than converting them into successes.
- Repaired the topology example's `TypeConfigExt` import and counted abandoned waiters from execution rather than a hard-coded number.
- Made topology preparation portable; original machine-specific scripts remain archived as evidence.

## Research boundaries

The simulator owns effects; the Wasm engine owns a suspended stack. Those do not automatically preserve the target's native worker topology, shared memory, allocator, runtime, kernel or filesystem. A killed waiter does not imply a cancelled operation. A process kill does not imply power loss. A private-memory component cannot silently replace shared-memory production coordination.

See the [rejection criteria](docs/history/continuation-kill-criteria.md) and [topology protocol](experiments/topology/protocol.md). Incomplete experiments are not future success claims.

## Provenance and licensing

[PROVENANCE](docs/PROVENANCE.md) describes snapshots, exports, hashes and exclusions. [THIRD_PARTY](THIRD_PARTY.md) identifies upstream material and retained license files. No project-wide license has been assigned to the original research code or analysis; upstream material retains its own licensing.
