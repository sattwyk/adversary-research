# Native production-topology observation

**This is an unfinished larger experiment.** The current example is a native WAL observation workload, not a deterministic cluster or power-loss harness.

The newer [controlled worker result](../../docs/research/topology-experiment-results.md) adds reproducible native checkpoints and historical regression probes. Use `scripts/prepare-controlled-topology.sh` followed by `scripts/run-controlled-topology.sh`; both default to `.work/controlled-topology`. Set `ADVERSARY_EXAMPLE=controlled_failure_probe` and a separate results directory to run the vote sync-error check. The native observation instructions below remain valid.

Read [the topology map](../../docs/research/production-topology.md) and [protocol](protocol.md) first. Preserving the actual worker, batch formation, callback timing and shared state is the point of the experiment.

## Restore and run

Prerequisites: Linux/WSL, Rust nightly-2025-12-11, a C/C++ build toolchain and protoc as needed by upstream dependencies. This larger build is separate from continuation CI.

```bash
rustup toolchain install nightly-2025-12-11 --profile minimal
bash scripts/prepare-topology.sh
bash scripts/run-topology.sh
```

Run those commands from the repository root. The preparation script restores pinned source archives into `.work/topology`, applies recorded tracked patches if nonempty, then overlays this example and the preserved Cargo.lock. It refuses to overwrite an existing directory.

`ADVERSARY_TOPOLOGY_DIR`, `CARGO_TARGET_DIR`, `ADVERSARY_TEST_DIR`, `ADVERSARY_RESULTS_DIR`, `ADVERSARY_TRIALS`, and `CARGO_BUILD_JOBS` can override paths/counts. On a Windows-mounted checkout, choose directories on the WSL filesystem for large builds.

## What the example does

For each trial it appends 256 deterministic entries through the actual Databend RaftLogStore, abandons selected completion receivers, waits for remaining callbacks and worker idle, closes normally, reopens, and compares decoded entries. It observes batch sizes through the upstream flush worker's debug log string.

The native worker still uses production scheduling/timing. Fixed workload input therefore does not imply fixed batches. A `recovered_equal=true` record checks graceful recovery for this workload, not crash durability or consensus safety. The logger's string matching is an observational probe, not a supported trace ABI.

## Original and cleaned code

The original example did not compile due to an incorrect `TypeConfigExt` import. That failure and its empty trials file remain in `results/2026-09-18/topology/`. The cleaned example imports the trait from `openraft::type_config` and derives the abandoned-waiter count rather than hard-coding it. Current execution status is in [STATUS](../../docs/STATUS.md).

No consensus or WAL algorithm is rewritten by the baseline example. The archived source snapshots remain pristine. The controlled experiment's separate patches add checkpoints and I/O wrappers; their scope and limitations are documented in the current-results report.
