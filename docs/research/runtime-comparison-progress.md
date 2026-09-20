# Runtime comparison: evidence ledger

20 September 2026. This is an interim evidence record, not a completion claim.

## Measured executor-hosting results

| Backend/configuration | Actual execution | Ten-trial wall time | Peak RSS | What controls the worker |
|---|---|---:|---:|---|
| Tokio current-thread/native | 10/10 recovery and within-run event-vector comparisons passed | 1.624 s | 34,784 KiB | Existing native checkpoint controller |
| Turmoil 0.7.2 | Same checks passed | 1.474 s | 35,356 KiB | Same controller; not Turmoil's task scheduler |
| Commonware 2026.9.0 default | Runtime stalled waiting on external work; exit 101 | No comparable completed run | — | Same controller |
| Commonware 2026.9.0 external | Same checks passed | 1.624 s | 34,804 KiB | Same controller; external mode permits native progress |
| MadSim 0.2.34 default | Simulator rejects production native thread creation | No completed run | — | Native worker forbidden by default |
| MadSim 0.2.34, native-thread opt-in | Same checks passed with `cfg(madsim)` | 1.624 s | 34,336 KiB | Same controller; native threads explicitly permitted |

Raw evidence: [`results/2026-09-20/runtime-comparison`](../../results/2026-09-20/runtime-comparison).
These are single unoptimized process runs, not performance distributions. Commonware
used a different toolchain. Ten trials per completed run recorded 64 recovered entries,
22 batches, 514 worker events, 1,988 partial-write calls and 26 sync calls per trial.
The initial runs compared full event vectors within a run and exported only
aggregates. The subsequent matrix below closes that cross-backend trace gap.
Native clock, file operations, worker internals between
checkpoints and producer scheduling remain outside complete deterministic control.

An additional [matched matrix](../../results/2026-09-20/runtime-matrix/verified-comparison.json)
now exports and compares complete traces. All **40/40 trials across four hosts** have
the identical 514-event worker trace, SHA-256
`0a7714f6622072e2568370673f7ee584b628c02d483d1fd7e9e7626976d0d7f3`.
All four also reproduce the injected vote-sync failure, including the same 17-event
trace: first and idle errors `StorageFull`, later vote error `Other`, one attempted sync.
The timeout watchdog is process-level in every host, replacing the original fixture's
Tokio-specific timeout. This is a test-harness change, not changed vote logic.

The [final-source revalidation](../../results/2026-09-20/runtime-matrix-final/verified-comparison.json)
repeats all eight cases after adding trace export and controller-attack configuration.
All 40 recovery trials and four sync-error checks pass again. Each result directory
includes the exact harness source, source hashes, compiler details, Cargo.lock and
invocation. This repeated check is validation of changed harness code, not an independent
benchmark sample selected to support a speed ranking.

The new runtime wrappers alter zero production Rust functions beyond the frozen
instrumentation patch. They add optional manifest dependencies and select an executor
around the same async workload. This establishes **hosting**, not equivalent distributed
fault coverage, native-storage correspondence, or independent runtime control of the WAL.

## Historical shared-cursor race

The real historical seek/read implementation fails under Shuttle after adding one
test-only checkpoint. The fixed pread implementation passes. The parent fails on
execution seven at level 2; both revisions pass all 5 level-0 and 19 level-1 executions.
The fixed revision completes 63 level-2 executions. The parent search stops at its first
failure, so seven is discovery cost, **not** its total state-space size.

The measured growth is 5 → 19 → 63 executions for this two-reader fixture and these
Shuttle scheduling rules. It does not predict growth for all application tasks. A
caller-boundary yield cannot split the faulty operation. A synchronization-only port
also has no internal reader lock to schedule. The coarsest demonstrated sufficient
checkpoint is between the two file operations; arbitrary instruction scheduling was
unnecessary. This extends coverage compositionally without a new runtime.

The [captured Shuttle schedule](../../results/2026-09-20/reader-boundary-replay/reader-schedule.txt)
reproduced the parent failure in **10/10 fresh process invocations**, one execution
each; [raw replay measurements](../../results/2026-09-20/reader-boundary-replay/replay.json).

## Native checkpoint counterexample

[`attack-checkpoint-gate.sh`](../../scripts/attack-checkpoint-gate.sh) changes only
the test workload: await the second append before submitting the third. With the
existing four-request gate the worker waits for requests that the suspended producer
has not submitted. The process exceeds the five-second watchdog. A one-request gate
completes and recovers all 64 entries in 0.321 s. Raw [quota-4](../../results/2026-09-20/checkpoint-gate/quota-4/usage.json)
and [quota-1](../../results/2026-09-20/checkpoint-gate/quota-1/usage.json) results preserve both outcomes.

This is a controller-induced liveness failure, **not** a production defect. The gate
is not observational: it assumes producer lookahead, adds synchronization edges and
can block a valid sequential client. The original 40 replay successes remain valid
for their pipelined workload, but do not establish a general deterministic scheduler.
A reusable controller must select enabled continuations without requiring future input
from a task already waiting on the blocked worker. Quota one is the comparison control,
not a proposed general fix: forcing every batch to one would lose the relevant batching
schedules. Native real-time batch deadlines remain another unqualified input.

## Capability dependencies established so far

| Scenario | Scheduler necessary for demonstrated reproduction? | Added semantic checkpoint | Storage model | Wasm | New runtime |
|---|---|---|---|---|---|
| Historical commit beyond last | No, sequential public API fixture | No | No, real file | No | No |
| Historical committed truncation | No, sequential public API fixture | No | No, real file | No | No |
| Historical conflicting purge identity | No, sequential public API fixture | No | No, real file | No | No |
| Historical zero-filled tail | No, recovery fixture | No | File damage fixture; not power-loss qualification | No | No |
| Abandoned waiter with successful recovery | Fixed controller provides repeatable worker trace | Existing worker hooks | Real short writes | No | No |
| Injected vote sync failure | No broad search; targeted failure | Sync adapter | Error injection only | No | No |
| Historical concurrent-reader race | Shuttle DFS for demonstrated controlled interleaving | One seek/read split | No, real file cursor | No | No |

The historical sequential regressions have not yet been rerun through every runtime
wrapper. Their lack of async dependencies suggests portability, but is not measured
matched coverage. No acknowledged-before-persistence mutant is claimed in this table.

## Remaining completion gates

- Run matched historical corpus cases in the viable hosts (sync-failure is now matched).
- Measure feature-disabled, passive and active instrumentation with comparable I/O.
- Demonstrate native-deadline sensitivity experimentally (checkpoint-induced blocking is established).
- Implement and compare three search strategies on the real worker with labeled mutants.
- Produce paired ordinary/semantic trace artifacts and independently check causal explanations.
- Finish measured LOC, effort and capability decomposition, then the requested 16-part report.

No result so far requires a separate Adversary runtime. That is a provisional finding,
not a substitute for the remaining exploration and fault-coverage experiments.

## Effort accounting

The repository records build start/end times and child-process resource usage. Commonware's
new-nightly build took 3m25s according to Cargo. The first reader compile failed because
the test module was private; a visibility correction made the six cases runnable.
The first MadSim build exhausted tmpfs before runtime compatibility was tested. A subsequent
build succeeded, but editing the shell runner while that invocation was reading it prevented
execution; that attempt is not a MadSim failure. Fresh invocations established the default
native-thread rejection and successful explicit opt-in. These
are integration costs, but they are not target semantic incompatibilities. Exact human
engineer-hours cannot be reconstructed from model/tool execution; no such hours are claimed.

The [integration LOC inventory](../../results/2026-09-20/integration-loc.json) counts physical
lines including comments and blanks. Executor selection and dispatch occupy 52 lines
across all four backends. The 241-line shared workload is primarily copied from the
existing controller probe; the 98-line sync-failure fixture preserves its preceding
oracle. The Shuttle test adds 92 lines and one production test-only checkpoint site
per historical revision. These figures must not be added to the frozen production
instrumentation count as though they were new consensus or WAL algorithm changes.

Primary runtime references: [Commonware external mode](https://docs.rs/commonware-runtime/2026.9.0/commonware_runtime/deterministic/index.html),
[Turmoil](https://docs.rs/turmoil/0.7.2/turmoil/), [MadSim](https://docs.rs/madsim/0.2.34/madsim/),
[Shuttle DFS and replay](https://docs.rs/shuttle/0.9.3/shuttle/).
