# Execution-topology experiment: evidence report, 20 September 2026

**Decision: retain the native worker-control experiment; do not claim validation of a separate deterministic-testing platform.** The real Databend storage path can run with cooperative checkpoints without replacing its worker. This is narrower than preserving a distributed Databend execution.

The requested complete experiment is **not completed**. This report closes the current evidence package, not the protocol. No multi-node safety result, matched competitor integration, power-loss correspondence, comparative exploration result, or human diagnosis measurement exists in this package.

## Reproducible observations

- [Native baseline](../../results/2026-09-19/native-topology/trials.jsonl): 20 workloads, 256 entries each, 20 different batch sequences; all graceful recovery checks passed.
- [Controlled native worker](../../results/2026-09-20/controlled-topology/trials.jsonl): ten repetitions, 64 entries each, identical worker-event vectors checked in memory. Each run recorded 22 batches, 514 worker events, 1,988 short-write calls and 26 sync calls. All entries survived graceful close/reopen. Full event vectors were not exported; aggregate logs and the assertion code are retained.
- [Vote sync failure](../../results/2026-09-20/controlled-sync-failure-final/trials.jsonl): injected first worker sync failure; save_vote and wait_worker_idle returned StorageFull; later save_vote returned Other after queue closure. One sync call. This establishes error propagation through the adapter, not a complete vote-safety proof.
- [Historical state cases](../../results/2026-09-20/historical-state-final/state.jsonl): exact parents accept three invalid operations; corresponding fixes reject them.
- [Historical zero-filled tail](../../results/2026-09-20/historical-zero-tail/938cc41.test.log): original ignored regression fails when enabled on parent 938cc41; [fix f71b906 passes](../../results/2026-09-20/historical-zero-tail/f71b906.test.log).

The controlled workload uses the real bounded std channel, native thread, byte quota, batch collector, vectored-write retry loop, file sync, shared persisted callback, per-request callbacks, rotation and recovery. It drops six result receivers; queued writes still finish.

## Reproduction

From the repository root on Linux/WSL:

```bash
export ADVERSARY_TOPOLOGY_DIR=/tmp/adversary-controlled-review
export CARGO_TARGET_DIR=/tmp/adversary-review-target
bash scripts/prepare-controlled-topology.sh
bash scripts/run-controlled-topology.sh
ADVERSARY_EXAMPLE=controlled_failure_probe \
  ADVERSARY_RESULTS_DIR=/tmp/adversary-review-sync-failure \
  bash scripts/run-controlled-topology.sh
bash scripts/run-historical-raft-log.sh
bash scripts/run-historical-zero-tail.sh
```

Preparation refuses an existing source directory. Databend uses nightly-2025-12-11. Historical runners clone upstream (or a supplied local Git repository), select exact commits, and preserve logs in results/current by default. Their dependencies are resolved at execution time; fully frozen transitive historical builds are not yet packaged. The initial Databend baseline lockfile is packaged; selecting the local patched chunked-wal updates its source entry.

Historical state values in the corrected JSON output are strings containing Rust debug representations. Earlier directories historical-raft-log, historical-raft-log-rerun, historical-raft-log-v2 and controlled-sync-failure retain failed builds or non-JSON debug output despite .jsonl suffixes. Use historical-state-final and controlled-sync-failure-final for machine-readable results.

## Historical corpus and reachability

| Fix | Defect and consequence | Observed evidence | Scheduling/effect requirement |
|---|---|---|---|
| [raft-log ec79536](https://github.com/drmingdrmer/raft-log/commit/ec79536de7a9d25d4b6da7523937b831f06138ef) | Commit beyond last and truncation of committed entries | Both parent-versus-fix cases reproduced | Sequential public API calls suffice; no simulator advantage demonstrated |
| [raft-log c3370c6](https://github.com/drmingdrmer/raft-log/commit/c3370c63affe7c4662c6bca7016984952dfd8e1f) | Conflicting purge identity produces inconsistent log state | Parent accepts; fix rejects | Real state checks and encoded replay matter; no worker scheduling required for this reproducer |
| [chunked-wal f71b906](https://github.com/drmingdrmer/chunked-wal/commit/f71b90615b8f39c1a751e58f7c33fbe6caf8e8dc) | All-zero successor after interrupted rotation prevents opening | Exact upstream regression fails on parent and passes on fix | File-length/data persistence distinction; reproduced by editing a file image, not a real power cut |
| [chunked-wal a485720](https://github.com/drmingdrmer/chunked-wal/commit/a485720121321a408e6cfbae00ec204e98245da2) | Recovery mutates damaged non-tail history before rejecting the chain | Source verified; fixed-version unit test passed in session; parent replay not performed | Real recovery/truncation and independent before/after file bytes |
| [chunked-wal 7bdcb55](https://github.com/drmingdrmer/chunked-wal/commit/7bdcb5520cee173473e73f81f3d35d453d29023f) | Worker error leaves completion waiters blocked | Source verified; current Databend sync-error propagation exercised; historical parent not run | Worker failure, completion state and waiter progress |
| [raft-log 707a9a2](https://github.com/drmingdrmer/raft-log/commit/707a9a2463ab215175ef9fbc8d86ea57c2aeac72) | Concurrent seek/read races access the wrong record | Source verified; fixed unit test passed in session; vulnerable parent not run | Schedule between seek and read across native readers; current worker-only hooks cannot do this |
| [raft-log aa6b279](https://github.com/drmingdrmer/raft-log/commit/aa6b279b2f288cb7e023aff7f300069367b63312) | Rotation hands old file to sync worker; successor never synced | Source verified; fixed unit test passed in session; parent not reproduced | File identity across rotation and sync; byte-vector single-file model would hide it |
| [raft-log 5a71534](https://github.com/drmingdrmer/raft-log/commit/5a71534afe46e17c48e13b4b184d49bac42a43a7) | No-sync request callback dropped | Attempt failed to compile: old public flush API lacks sync argument | Internal worker request fixture needed. Retained harness is not a working historical reproducer |

These are four reproduced scenarios across three fixing commits, not five reproduced historical bugs. They are not established production incidents. In particular, callers supplying invalid commit/purge/truncate operations are storage API probes; reachability from a correctly operating OpenRaft core has not been shown. The purge check was subsequently removed by pinned raft-log 0.4.6; the historical fix is not evidence that the pinned current version promises that validation. No mutants are counted.

## 1. Did the real production execution topology survive?

**For the exercised storage slice, yes; for the complete distributed target, unproven.** The actual native worker, queue, shared callback and foreground producer remain separate contexts. Rotation still divides batches. The fixed controller gates worker receive until requests are enqueued and injects selected batch timeout outcomes.

Important limitation: std::time::Instant and recv_timeout remain native. The workload extends the batch window to 30 seconds, with controller timeout decisions. A long host pause can still change the path before a checkpoint. Only worker events are compared; producer interleavings, lock acquisition order and OS effects are not globally replayed. Therefore ten matching traces do not establish a general deterministic scheduler.

The process-wide controller also lacks node/lifetime identity. It cannot yet distinguish multiple simultaneous WAL owners. No task executor, snapshot worker or protocol transport was integrated.

## 2. Exactly what production code changed or disappeared?

The [chunked-wal patch](../../experiments/topology/patches/chunked-wal-adversary.patch) changes four existing files and adds one integration module:

| File | Added | Removed | Classification |
|---|---:|---:|---|
| Cargo.toml | 5 | 0 | Experiment feature |
| src/lib.rs | 1 | 0 | Module export |
| src/wal/flush_client.rs | 35 | 0 | Checkpoints plus request classification helper |
| src/wal/flush_worker.rs | 94 | 7 | Checkpoints and wrappers at existing write/sync boundaries |
| src/adversary.rs | 226 | 0 | New controller API and native fallbacks |

Six existing functions change: FlushClient::send; FlushWorker::run, run_inner, collect_write_batch, sync_data_files; write_all_vectored. Two request classification helpers are added. No async/CPS conversions or ownership migration. The patch contains 26 checkpoint call sites (countable with rg), not 26 independent scheduling dimensions.

Databend changes one Cargo.toml dev-dependency line; no production Rust functions change. OpenRaft, raft-log, record codecs and recovery algorithms are unchanged. New examples, scripts and the 226-line controller module are integration code, not preserved production code. No generated bindings.

Semantics under control deliberately change at timed receives and injected I/O outcomes. The default controller methods invoke native file calls. No-controller compilation and unit tests passed, but instrumentation overhead was not benchmarked.

Excluded from exercised behavior: actual Raft core/elections/replication, tonic framing/TLS/DNS, request retry/dedup, state-machine application, snapshots/rotbl, multi-reader interleavings, process generations, kernel/filesystem internals and power-loss persistence. Their source remaining in the dependency tree is not coverage.

## 3. Which historical/injected safety bugs were representable?

The corpus above supplies concrete historical replay evidence. Short writes are exercised through the actual production IoSlice advancement loop. Dropped result waiters do not retract operations. Sync failure crosses real Databend callback/oneshot and worker-state boundaries.

This supports boundary observability for durability gating, recovery and rotation. P1/P3 surviving-quorum safety, P2 incompatible votes after restart, P4 snapshots and P5 retry/dedup were not checked end-to-end. No client-success durability violation was discovered.

## 4. Which important bugs were impossible to represent?

The current harness cannot deterministically schedule concurrent seek/read defects outside the worker, arbitrarily kill one native worker without cleanup, cut machine power, model rename/directory-sync states, or control node elections and network retries. These are limitations of this implementation, not proofs of impossibility for all semantic schedulers. Covering the read race requires justified read/synchronization checkpoints, not declaring it irrelevant.

## 5. Did suspended Wasm provide a measurable integration advantage?

No. Wasm was not used in this target experiment. Native checkpoints preserved the worker without moving its shared payload-cache callback across stores. Earlier redb suspension evidence remains valid for isolated synchronous owners. For Databend, make Wasm optional; no measured comparison justifies calling it superior or conclusively more expensive.

## 6. How did Commonware compare?

No equivalent Commonware integration was executed. Earlier interface/source review is retained in the continuation report, but cannot establish comparative coverage, effort or replay. The native controller is separable from any runtime and could be contributed as a backend or bridge. Competitor dominance is unmeasured.

## 7. How did Turmoil compare?

No matched runtime integration was executed. Existing filesystem work must be included in any future comparison. Native-thread gating and modeled persistence remain distinct requirements; this report supplies no measured advantage over Turmoil.

## 8. How did MadSim compare?

No matched runtime integration was executed. Source observations about generic filesystem behavior are not a benchmark. No claim about half the integration effort or equivalent failure coverage is supported.

## 9. How did actual-process testing compare?

The baseline executes native files and the actual worker in a process and observes nondeterministic batches. It is not a multiple-node actual-process baseline. SIGKILL/restart, socket partitions and crash-device experiments were not run. Graceful Drop invokes production shutdown; it is not process death.

## 10. Did semantic exploration improve bug discovery?

Not measured. The controller runs one fixed schedule with repeat checks. Random, perturbation, bounded-delay/PCT, coverage-guided and semantic-targeting search were not implemented. There are no equal-budget distributions or code-coverage results. Historical API reproducers demonstrate bugs ordinary tests can expose without exploration. Remove superiority from current claims.

## 11. Did semantic traces improve diagnosis?

Not measured. Events distinguish write, sync, persisted callback and request callback, but there is no causal trace tying replicated entry identity to client success, no paired failure diagnosis, and no blinded human study. The current trace omits per-write file identities and lifetimes. Event readability alone is not diagnostic-effectiveness evidence.

## 12. Did native storage outcomes fit the model?

No correspondence relation was tested. Writes and syncs use the native filesystem; short writes and one sync error are injected. No abstract durable-state model exists in this new harness. File-image zeroing is a recovery input, not evidence that this deployment produced that crash state. No ext4/XFS/NVMe profile is qualified.

## 13. Actual engineering cost breakdown

Command observations: initial controlled build 18.51 seconds with reused cache; final ten-trial command 8.38 seconds including a 6.20-second rebuild; separate crate unit-test builds 35.95 seconds (chunked-wal) and 43.92 seconds (raft-log), with concurrent package-cache contention. These are not comparable throughput measurements.

A reliable category-by-category active engineering ledger was not maintained during the repeated interrupted turns. Human engineer-hours cannot be reconstructed from command duration, conversation timestamps or file mtimes. Costs of source understanding, adapter design, competitor integration and native validation are therefore **not measured**, not zero. This failure of measurement prevents an economics conclusion.

## 14. Which kill criteria fired?

| Criterion | Evidence-based disposition |
|---|---|
| K1 topology replacement | Not triggered for the exercised worker; full target untested |
| K2 correctness rewrite burden | Not triggered in the storage slice; total integration unknown |
| K3 competitor dominance | Untested |
| K4 Wasm irrelevance | Make Wasm optional for this prototype; comparative cost/benefit not measured |
| K5 Wasm distortion | No Wasm target run to assess |
| K6 scheduling insufficiency | Current worker-only checkpoint set excludes the historical concurrent read race; expand justified read synchronization before claiming that coverage |
| K7 storage correspondence | Untested; no qualified profile may be claimed |
| K8 exploration non-result | Superiority unsupported and removed; no experiment establishes equality or inferiority |
| K9 diagnosis non-result | Superiority unsupported and removed; no human evidence |
| K10 economics | Untested; neither cheap portability nor months of required work established |

Absence of an experiment is not a passed kill criterion or a measured negative outcome.

## 15. Should Adversary remain a separate architecture?

The evidence does not justify a separate runtime platform. Retain this as a native instrumentation and conformance research project that can attach to an existing deterministic runtime. Keep the effect interface independent of execution backend, add explicit owner/lifetime and file identity, and control clocks and read/synchronization points before promising replay. A storage-only success cannot justify language-neutral SDKs or a production-correspondence claim.

## 16. The strongest hypothesis that survives the experiment

For Databend's isolated RaftLogStore workload, a small native instrumentation patch can preserve the real WAL thread, queue, batching, rotation, encoding, callbacks and recovery while repeatedly selecting the same worker event sequence and injecting short-write/sync-failure outcomes. Whether this extends to real multi-node topology, qualified crash states, competitive integration cost or better search and diagnosis remains unresolved.
