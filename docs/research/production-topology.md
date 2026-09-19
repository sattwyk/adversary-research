# Production execution topology — pre-integration record

Source inspection completed before production edits. This is a topology and contract record, not evidence that deterministic integration has succeeded. Experiment uses local public-source checkouts and synthetic reliability workloads only.

## Source pins

| Component | Revision/version |
|---|---|
| Databend Meta | `4b7bba5c37864441fb451e65bc6ec9a91fb852e7`, September 15, 2026 |
| raft-log | `55a42ae254b984532204a8787646e78471ddd8f4`, 0.4.6 |
| chunked-wal | `6014c3f1ba601e855e2f50cef91e923a884384fe`, 0.2.3 |
| OpenRaft | crates.io 0.10.0-alpha.29, exact Databend direct dependency |

The resolved transitive runtime is openraft-rt-tokio 0.10.0-alpha.34. The experiment must preserve the generated Cargo.lock rather than assuming all OpenRaft packages have the same prerelease version. Baseline compilation succeeded on Linux with Databend's pinned nightly-2025-12-11 and experiment-only `RUSTFLAGS='-C linker=cc'`. The repository's mold linker and an attempted lld override were unavailable. These are build-environment failures, not architecture failures.

## Execution graph

```text
client request worker / retry loop (Tokio tasks, own runtime possible)
   | same serialized transaction may be retried after transient RPC error
   v
tonic/h2 sockets -> server request task -> forwarding/retry -> OpenRaft API
                                                               |
            tick / heartbeat / vote / replication tasks <------> Raft core task
                       |                                      |        |
                       |                                      |        +--> SM command queue
                       |                                      |              |
                       |                                      v              v
                       |                              RaftLogStore       SM worker task
                       |                              Tokio RwLock       apply/install
                       |                                      |              |
                       |                         raft-log encode/apply       +--> client responders
                       |                                      |                   after apply commit
                       |                          chunked-wal pending bytes
                       |                                      |
                       |                        byte quota Mutex/Condvar
                       |                         bounded sync_channel(1024)
                       |                                      |
                       |                                      v
                       |                            native flush thread
                       |                            recv / recv_timeout
                       |                            collect WriteBatch
                       |                            writev + short-write loop
                       |                            sync_data file(s)
                       |                            persisted-cache callback
                       |                            per-request callbacks
                       |                            quota release + progress
                       |                                      |
                       +------ IOFlushed watch / vote oneshot <--+

SM BuildSnapshot command -> independently spawned snapshot-builder task
   -> writer permit / frozen map -> async stream -> bounded Tokio mpsc
      -> spawn_blocking snapshot writer -> real rotbl DBBuilder
         -> commit temporary file -> rename final snapshot -> open DB
   -> publish compacted state -> cleanup old snapshots

background compactor task -> same current-state slot + writer permits
incoming snapshot stream -> blocking receiver/file sync -> rename -> install permit

rotation foreground: enqueue predecessor sync -> create successor + directory sync
                     -> enqueue AppendFile -> encode checkpoint -> enqueue write
worker FIFO: predecessor write/sync -> file switch -> successor checkpoint write
```

## Ownership, concurrent progress and ordering

| Path / owner | Other progress while blocked | Shared state | Required order / completion |
|---|---|---|---|
| Raft core append future | Other Tokio tasks and native flush worker; a synchronous quota/channel block occupies that Tokio worker thread | `Arc<tokio::sync::RwLock<RaftLog>>`; pending bytes and Raft-log logical state | Mark IO submitted before calling storage; encode/append/queue before return; callback is separate from return |
| Vote persistence future | Other tasks and flush thread; core awaits oneshot after dropping log write guard | Same log store; vote record and callback channel | Save vote record, queue sync, successful worker callback, then save_vote return and dependent protocol response |
| Native flush worker | Raft tasks may enqueue more work; foreground reads can progress subject to locks | Request queue, `WorkerState` Mutex/Condvar, byte quota Mutex/Condvar, atomic metrics, shared file handles/cache | Batch by size/deadline/non-write boundary; write all vectors, sync when requested, persisted callback, per-request callbacks, quota release, progress publication |
| Rotation foreground under log write guard | Flush worker processes older requests | Current/open and closed chunk bookkeeping; worker file list is separately owned | Predecessor sync is enqueued even when pending data is empty; successor name is directory-synced; AppendFile separates batches; checkpoint uses worker queue |
| Recovery during open | No new worker until replay completes; other nodes can run | Directory lock, files, reconstructed RaftLog state/cache | Scan/replay real records; incomplete tail handling; chain validation; start worker only after recovered state established |
| State-machine apply worker | Snapshot task/compactor/network/Raft core may progress | Current `Arc<StateMachine>` behind a std Mutex; writer permit; map views and pending changes | Acquire writer permit; apply real commands; commit application changes; send client responders; notify Raft core |
| Snapshot builder + blocking writer | Protocol/apply progress where writer permit and immutable views permit it | Frozen views, compactor permit, bounded mpsc, persisted snapshot DB | Build actual DB from stream; Finish record commits; rename and open; publish view then cleanup |
| Snapshot installation task | Concurrent readers/watch registrations/compactor; ordinary SM commands are serialized by SM worker | Current-state slot and writer permit | Acquire permit before reading current instance/freshness and swapping state; historical fixes show the slot/permit ordering is significant |
| Client/server RPC tasks | Other requests, independent retries, replication and storage continue | Endpoint rotation state, connection pools/streams, Raft channels | Serialization, send, remote effect and response are distinct; RPC cancellation does not imply undoing submitted Raft/WAL work |

The WAL worker's `on_persisted` callback also takes the raft-log payload-cache `std::sync::RwLock` and makes closed durable payloads evictable. The worker does not own an isolated state machine with no shared memory. Moving only it into a separate Wasm store requires an explicit bridge for this callback and shared ownership; pretending it is a standalone `durable_append` service changes the topology.

## Cancellation and lifetime

1. **Caller abandons result:** dropping a vote oneshot receiver or RPC response waiter does not remove the already enqueued write. Callback send may fail while storage still succeeds.
2. **Queued operation withdrawn:** the current WAL public queue has no general withdrawal API. The model must not invent one. Cancellation before submission differs from cancellation after enqueue.
3. **Task aborted:** aborting an async task can drop its future/guards, but synchronous calls must return or yield before cancellation takes effect. A started Tokio blocking closure is not automatically stopped by aborting its join handle.
4. **Connection closed:** tonic stream cancellation may stop producer/consumer tasks. Already accepted Raft work and queued WAL writes can continue. Retry must be a new attempt, not rollback of the old attempt.
5. **Graceful shutdown:** `ChunkedWal::shutdown` submits final synchronous pending bytes, closes sender, joins worker and checks progress/error. `Drop` calls shutdown. The runtime has a separate native owner thread waiting for shutdown and dropping its Tokio runtime.
6. **Process termination:** no user callbacks or continuations execute after death. Kernel-accepted writes can outlive the process; process restart on the same kernel retains page cache. A simulator must invalidate continuation generations without running graceful Drop as a crash surrogate.
7. **Machine/storage failure:** volatile storage effects may disappear under the declared profile. This is a separate event, never inferred from a killed process.

## Time, entropy and platform escape points

- OpenRaft obtains spawning, channels and time through its `AsyncRuntime`; Databend's concrete `TypeConfig` chooses `TokioRuntime`.
- Databend's `SpawnApi` abstracts some spawning, but returns concrete Tokio JoinHandles and tonic Channels. Changing one generic runtime parameter is not enough to control the full program.
- WAL batching uses `std::time::Instant`, blocking `recv_timeout`, std threads, Mutex/Condvar and byte-quota blocking. These determine which requests share a sync, and therefore are correctness-relevant scheduling inputs.
- Snapshot temporary-name construction uses SystemTime and a thread sleep. Leader timestamps in `LogEntry.time_ms` drive expiry consistently on replicas; they are part of application behavior, not merely diagnostics.
- Election timeout randomness and runtime clocks must be routed through the controlled runtime; all other entropy sources need an implementation-time dependency audit. Source inspection here does not certify absence of hidden entropy.
- File APIs include create/open, append writes, vectored writes with partial-write handling, positional reads, metadata/inode identity, truncation/recovery, file data sync, directory sync, rename and unlink. The model must include file identity separately from pathname.
- Network uses actual protobuf/legacy serialization and tonic/h2 streaming plus TCP, DNS and reconnect paths. A direct typed OpenRaft message transport would exclude serialization/framing/reconnect behavior and must not be counted as equivalent coverage.
- Snapshot receiver explicitly syncs the received temporary file. The wrapper's rename alone does not establish all filesystem guarantees: the rotbl builder and native filesystem profile must be checked before any publication-correspondence claim.

## Correctness checker contract before workloads

- **P1:** use unique immutable logical keys and an independent durable client-history journal. A successful write must be observable after recovery/election of a legal surviving quorum. For a three-voter cluster, test all-node power interruption with acknowledged durable media retained, or permanent loss of at most one node with the other two recovering according to the profile. Do not count arbitrary destruction of a quorum's durable media as a Raft bug.
- **P2:** record protocol vote requests/responses and independently decode recovery metadata; a restarted node must not grant incompatible votes in the same term. A delayed response for an obsolete lifetime cannot be mistaken for a new grant.
- **P3:** track successful logical operation identities and externally verified recovered effects, not merely OpenRaft's volatile committed metric. Inspect log/snapshot recovery jointly: compaction legitimately removes log records.
- **P4:** repeat P1/P3 across interrupted rotation, snapshot publication and log purge. Keep actual encoding and snapshot builder/recovery paths. An isolated WAL test establishes only the WAL part of this property.
- **P5:** current `LogEntry` has time and command, no persistent transaction ID. Client transaction loops clone and resend requests on retryable RPC status. Do not assume universal exactly-once effects. Use application conditional transactions with a request-marker key to define a legitimate idempotency invariant; separately record native unconditional-retry behavior. The internal tracing request counter is not evidence of durable deduplication.
- **P6 (liveness/error propagation):** after permanent storage failure or callback panic, queued senders and waiters must resolve/fail rather than hang indefinitely. This is a documented worker regression class, kept separate from data-loss safety results.
- **P7 (recovery non-destruction):** rejected corruption in an older non-tail chunk must not be silently repaired by truncating unrelated durable history. Use the documented historical regression and compare exact file bytes before/after rejected open.

## Source anchors

Paths are relative to the pinned repositories:

- Databend: `crates/server/raft-log/src/{store,impl_raft_log_storage,callback,codec_wrapper}.rs`; `crates/server/types/src/{raft_types.rs,log_entry/mod.rs}`; `crates/common/runtime-api/src/{lib,tokio_impl}.rs`.
- Databend: `crates/server/state-machine/src/{store,state_machine,impl_raft_state_machine}.rs`; `crates/server/snapshot-store/src/{writer,receiver}.rs`; snapshot-config's publication implementation.
- Databend: service `network.rs`, `meta_node/meta_node_builder.rs`, `meta_node.rs`, `raft_listener.rs`; `crates/client/client/src/{grpc_client,rpc_handler}.rs`.
- chunked-wal: `src/wal/{flush_worker,flush_client,queued_bytes,chunked_wal,write_batch}.rs`, `src/config.rs`, and chunk creation/recovery code.
- raft-log: `src/raft_log/raft_log.rs` and its payload-cache/state-machine code.
- OpenRaft: `src/core/raft_core.rs` command execution, `src/core/sm/worker.rs`, `src/storage/callback.rs`, type-config/runtime interfaces.

No deterministic-preservation conclusion is drawn yet. This record authorizes the next experimental boundary extraction without simplifying the worker algorithm.
