# Matched runtime and reader-boundary experiments

This experiment continues the [user's objective](../../docs/briefs/runtime-comparison-objective.md).
It is not the full Databend cluster integration. The experimental target is frozen
at research commit `7a53b559757bfa3892259079f707466b524a5def` and the upstream pins
recorded in the [topology report](../../docs/research/topology-experiment-results.md).

## Executor comparison

`shared.rs` preserves the preceding controlled Databend storage workload. The only
initial change was making its async entry point public and moving executor startup
to `runtime_probe.rs`. The native WAL thread, queue, real filesystem, encoders,
batching loop, callbacks and recovery remain in production code. The existing
checkpoint controller still decides worker progress for every backend.

```bash
bash scripts/prepare-runtime-comparison.sh
ADVERSARY_BACKEND=native bash scripts/run-runtime-comparison.sh
ADVERSARY_BACKEND=turmoil bash scripts/run-runtime-comparison.sh
ADVERSARY_BACKEND=commonware ADVERSARY_TOOLCHAIN=nightly bash scripts/run-runtime-comparison.sh
ADVERSARY_BACKEND=commonware-external ADVERSARY_TOOLCHAIN=nightly bash scripts/run-runtime-comparison.sh
ADVERSARY_BACKEND=madsim bash scripts/run-runtime-comparison.sh
```

Preparation refuses an existing target directory. Set `ADVERSARY_TOPOLOGY_DIR` for
a fresh location. `ADVERSARY_RESULTS_DIR` selects an absolute output directory.
Build products use the Linux user's cache directory; `/tmp` is a small tmpfs in
the experiment environment. Run backends sequentially because their example
binary shares a target path. Compilation is excluded from process measurements.

Each successful run performs ten independent close/reopen trials, compares all
64 decoded entries with the submitted values, and checks complete worker-event
vectors against the first trial in memory. JSONL exports aggregate observations;
matching exported counts alone do **not** establish cross-backend trace equality.
`scripts/run-runtime-matrix.sh` exports full traces and verifies equality across
all four hosts with `verify-runtime-matrix.py`. It also runs the same vote-sync
failure injection in each. There is no simulated power cut or multi-node consensus
execution in this probe.

Commonware 2026.9.0's default deterministic executor stalls on this native-worker
future. Its `external` feature is an explicit supported configuration for external
progress, and passes this workload. The successful run required a newer compiler
than Databend's pinned nightly because `sysinfo 0.39.6` requires Rust 1.95.
This compiler difference precludes treating these one-off timings as a controlled
runtime performance ranking.

MadSim is built with `--cfg madsim`; merely depending on the crate would execute its
non-simulator facade. `ADVERSARY_ALLOW_SYSTEM_THREAD=1` explicitly opts into MadSim's
native-thread escape hatch. It must never be described as controlled thread execution.
The default configuration was executed and rejected native-thread creation. With
the opt-in, all ten trials and the matched failure probe passed.

`scripts/attack-checkpoint-gate.sh` demonstrates a controller limitation: awaiting
the second append deadlocks the four-request gate. A one-request gate completes
the same client workload. The five-second timeout is an expected **harness** failure,
not a Databend bug. This test must remain separate from replay-success reporting.

## Historical reader race

```bash
bash scripts/run-reader-boundary.sh
```

The historical parent and fix are separate clean worktrees around raft-log commit
[`707a9a2`](https://github.com/drmingdrmer/raft-log/commit/707a9a2463ab215175ef9fbc8d86ea57c2aeac72).
The test retains actual file handles, seek/pread, record decoding, cache behavior,
and native WAL setup. Two reads address different entries in the same closed chunk.
The parent gains **one test-only call site after seek**; the fix gains one before
pread. Only the two designated reader tasks yield there. Shuttle 0.9.3 controls
those tasks; the WAL worker is idle during the race and remains a native thread.

Level 0 has only Shuttle spawn/join scheduling. Level 1 adds a caller-boundary
yield. Level 2 adds the targeted seek/read checkpoint. This is a compositional
Shuttle experiment, not evidence of Adversary controlling all native synchronization.
On the fixed implementation DFS completes 5, 19 and 63 executions respectively.
The parent has no failure at levels 0/1 and fails on the seventh level-2 execution.
The script additionally replays the captured schedule ten times.

The first reader build failed on test-module visibility; that log is retained in
`results/2026-09-20/reader-boundary`. `reader-boundary-v2` contains the first successful
six-case comparison. Later replay results are stored separately. Do not classify
compilation or environment failures as target defects.

Loom does not model a kernel-owned file cursor. A Loom atomic-cursor model would
be a separate abstraction, not execution of this production reader. Shuttle can
retain the actual file operations because the targeted yield is between them.
