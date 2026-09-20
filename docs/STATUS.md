# Evidence status — 20 September 2026

## Matched runtime and concurrency results

See the [runtime comparison evidence ledger](research/runtime-comparison-progress.md).
Native, Turmoil, Commonware external mode and MadSim with native-thread opt-in now
pass 40 matched recovery trials with identical exported worker traces. Each host
also propagates the same injected vote-sync failure. The controller retains ownership
of worker scheduling; these results establish hosting, not a fully simulated cluster.

Shuttle reproduces the historical shared-file-cursor race with one targeted checkpoint
and replays it 10/10 times. The fixed reader passes all 63 explored schedules. A separate
negative test shows the current four-request controller gate can deadlock a valid client.
Matched historical corpus runs, overhead measurements and comparative exploration/diagnosis
remain unfinished. No separate runtime advantage has been demonstrated.

## New topology evidence

See the [complete current-results report](research/topology-experiment-results.md) for scope, source-change counts, historical commit links and the 16-question assessment.

- Ten controlled native-worker trials recovered 64 entries each with identical in-memory worker-event vectors. Each trial recorded 22 batches, 514 events, 1,988 short-write calls and 26 sync calls.
- A first-sync StorageFull injection propagated through Databend save_vote and worker-idle wait; later save_vote failed promptly.
- Four historical failing scenarios were reproduced across three fixing commits: commit beyond last, committed truncation, conflicting purge identity, and zero-filled tail recovery.
- The controlled preparation patches apply to fresh source archives.

These are storage-slice results. Native clock expiry, producer interleavings and multiple owners are not fully controlled. This is not a general deterministic cluster or a finished execution-topology experiment.

## Completed and revalidated

| Check | Evidence | Outcome |
|---|---|---|
| Host/guest formatting and host Clippy | `scripts/run-probes.sh` on WSL, Rust 1.97.1 | Passed, warnings denied for host code |
| Synchronous component probe | [WSL stdout](../results/2026-09-19/wsl-continuations/component-results.txt) | Four schedules, 101 identical traces each |
| Real redb commit/recovery probe | [WSL stdout](../results/2026-09-19/wsl-continuations/redb-results.txt) | Nine crash scenarios, ten identical traces each; expected recovery assertions passed |
| Native fiber comparison | [WSL stdout](../results/2026-09-19/wsl-continuations/native-fiber-results.txt) | 100 repetitions; application destructors run when a native coroutine is dropped |
| Original redb dependency integrity | [Original integrity record](../results/2026-09-18/continuations/integrity.txt) | 71 packaged files checked, zero mismatches; registry archive matched Cargo.lock |
| Original Databend storage package baseline | [Original test log](../results/2026-09-18/topology/baseline-tests.log) | 11 unit + 1 recovery integration test passed |
| Repaired native topology example | [Build](../results/2026-09-19/native-topology/build.log), [trials](../results/2026-09-19/native-topology/trials.jsonl) | 20/20 graceful close/reopen trials recovered all 256 entries, including writes whose waiters were abandoned |

The native topology observation recorded **20 distinct batch sequences across 20 identical-input trials**. This is evidence that this native worker execution is not deterministic from workload input alone. It is not a discovered production bug or a proof that a specific simulation boundary cannot control it.

The topology example was repaired during repository cleanup: `TypeConfigExt` comes from `openraft::type_config`, and the abandoned-waiter count is computed. Upstream algorithms remain unchanged. Original failed logs and the empty original trial file remain archived.

The continuation probes also ran previously on Windows. Separate successful Linux/WSL runs do not establish general bitwise cross-platform replay or equivalent native filesystem behavior.

## Still unfinished or untested

- Full Databend multi-node deterministic integration preserving the actual worker topology.
- Full distributed-runtime comparisons beyond the completed storage-slice hosting matrix.
- Broader historical bug rediscovery through the distributed target and an independently labeled mutant corpus.
- Qualified filesystem/device power-loss correspondence.
- Comparative exploration throughput and diagnostic effectiveness.
- Guest threading, shared-memory/native-runtime fidelity, and full WIT/component integration for redb.
- General-purpose replay/snapshot/exploration infrastructure.

The native topology example uses graceful close. It does **not** establish process-kill or machine-power-loss behavior. The original native-cluster build log does **not** establish a completed cluster workload.

## CI

The GitHub workflow runs the completed continuation checks on Linux and uploads result logs. It does not run or claim completion of the larger topology objective. Local revalidation evidence above is independent of the eventual hosted workflow result.
