# Adversary: continuations, effects, and production correspondence

Research and experiments as of **18 September 2026**. This review addresses the follow-up continuation hypothesis and the original RFC. Conclusions distinguish executed probes, inspected implementations, and proposed experiments. The current RFC's explicit request/completion reactor is not silently credited with capabilities demonstrated by a different execution backend.

## 1. Does the continuation hypothesis survive?

**Yes, narrowly. H2 is the best-supported answer: suspended Wasm imports expand the synchronous production code that can run under deterministic control. They do not preserve an arbitrary production runtime or its concurrency.** This is a material correction to the claim that the architecture must always collapse into an async runtime plus a sandbox. It is not evidence for a general language-neutral replacement for deterministic native execution.

I built and executed three probes, rather than inferring feasibility from API names:

| Executed probe | Observation | What it does not establish |
|---|---|---|
| Wasmtime 48.0.2 components, two Stores, synchronous guest imports | Nested call frames and locals survived short writes and suspension. The host interleaved nodes and separately chose effects, persistence, completion and kill. Four schedules each produced 101 identical event traces. | Production application preservation, guest thread scheduling, cross-platform replay, throughput. |
| Unmodified redb 4.1.0 compiled as a core Wasm module | A real transaction committed and read 42. The driver killed suspended calls halfway through each of four commit writes, with two persistence choices, then reopened a fresh instance. Eight write-crash cases recovered the old state. A ninth case persisted sync but killed before returning completion and recovered 42. Each crash scenario replayed ten times. | Full component/WIT integration, native file-backend semantics, concurrent redb readers, real filesystem conformance, broad corruption coverage. |
| Native corosensei 0.3.4 fibers | Nested native Rust and `std::io::Write::write_all` handled short writes under host-selected resumption. One hundred repetitions passed. Dropping a suspended coroutine ran application destructors. | Abrupt process death, native TLS/global-state isolation, full deterministic execution. |

The redb guest wrapper is 54 lines, the redb host driver 114, the component probe 150, and the native fiber probe 43. These are proof scripts, not estimates for a production SDK. **Zero redb source files were changed:** all 71 files in the registry archive were compared with the compiled sources, and the archive matched Cargo.lock. The storage model is a pair of byte vectors, not an ext4/XFS/device emulator.

The redb recovery oracle was restricted to accepting an absent table or the expected value; unexpected storage/table errors fail. The guest Drop marker runs normally but does not run on the tested suspended-call kill. Host cleanup still runs. The redb probe is **core Wasm**, whereas the smaller mechanism probe is a **component**. Combining those into a full production component build remains an experiment.

This evidence rejects two overly strong objections: that synchronous stacks necessarily require handwritten reactors, and that current Wasmtime cannot give the host deterministic control while those stacks wait. It does not reject the much stronger concern that important concurrency disappears when a whole guest is suspended.

Reproduction source, lockfiles, commands, scope notes and recorded stdout are in `continuation-probes/`. No new production bug or performance result is claimed.

## 2. Is the effect boundary real?

**It is real as a contract about observable histories, not as a universal inventory of operating-system behavior.** The useful condition is that every influence on a target invariant crosses a controlled boundary, or belongs to a deliberately atomic region justified for that invariant.

For one isolated execution context with private volatile memory, a fail-stop crash during pure computation can often be moved to an adjacent external-effect boundary without changing the surviving world. The interrupted stack and private heap disappear either way. The simulator must retain the same outstanding operations, external clock history, messages and durable bytes. This is a conditional equivalence argument, not a theorem about all software.

| Crash location | Effect-boundary representation | When it fails |
|---|---|---|
| Inside serialization, before bytes escape | Usually equivalent to no submission | Output is streamed incrementally; shared memory or asynchronous I/O already references the buffer. |
| After private metadata changes, before the next I/O | Usually equivalent if the entire owning process dies | Another surviving thread/process observed that metadata; only one task is cancelled. |
| Between two synchronous I/O calls | Representable if each call can suspend | The current reactor treats the enclosing call as indivisible. |
| During a write or sync | Representable through partial effects and persistence decisions | The model omits relevant atomicity, error, metadata or ordering rules. |
| Inside a critical section | Whole-process death often reduces to external effects | Shared locks/state survive, robust-mutex recovery matters, or other contexts continue. |
| Inside allocator/runtime code | Process death may reduce to effects | The defect is allocator corruption, a GC race, finalization, or thread-local cancellation rather than process death. |

Consequential included classes are real: acknowledgment before durability; truncated records; recovery choosing the wrong generation; retry after lost reply; duplicate application request; stale term/lease; timeout versus reply; reconnect and cancellation races; and process death with requests already accepted elsewhere. They require preserving the actual codecs, recovery, deduplication, consensus and transaction paths above the effects.

Consequential excluded classes are also real: a lock-free publication defect; SQLite's competing shared WAL-index updates; an io_uring buffer lifetime error; filesystem rename/directory persistence assumptions; native allocator corruption; or a GC/runtime bug. These are not all merely performance gaps. The correct denominator is a curated corpus of safety regressions from the target projects, not all imaginable bugs or a guessed percentage of incidents.

**Storage remains the principal correspondence risk.** Submission, effect, completion and durability should remain distinct. A write may affect bytes before its waiter resumes; a sync may become durable before its completion is delivered; process death need not erase kernel/device work. But a 512-byte persistence unit is a model parameter, not a hardware fact. File length, overlapping writes, namespace changes, directory sync, error-after-partial-effect, writeback errors, truncation, buffered/direct I/O and successful sync guarantees need named profiles. The RFC's serialization of mutations to an inode and flat-root restrictions are useful scope limits only if target software really obeys them.

Conformance means checking **trace inclusion and allowed outcomes**, not demanding that a nondeterministic real filesystem reproduce every simulator schedule. Use small litmus programs on declared filesystem/mount/kernel/device combinations; record syscalls/completions and resulting recovery images; reject native outcomes absent from the abstract profile; validate simulator counterexamples against native failpoints or a VM where possible. Application-level syscall logs alone cannot tell which bytes survived sudden power loss. Block-level crash testing or equivalent controlled infrastructure is needed for that claim. Bounded crash testing has repeatedly found serious bugs with small workloads, supporting this qualification strategy rather than eliminating it. [CrashMonkey/Ace, OSDI 2018](https://www.usenix.org/conference/osdi18/presentation/mohan)

Do not enlarge one abstract filesystem until it impersonates every platform. If one target needs mmap coherence, file-lock subtleties, direct-I/O alignment and asynchronous device queues to explain its bugs, a lower backend is likely cheaper.

## 3. Is the continuation boundary real?

**Yes. It is the ownership and suspension boundary of an execution context. It is distinct from the effect API, and neither is sufficient alone.**

| Architecture | Naturally preserves | Main change required | Irreducible limitation |
|---|---|---|---|
| A. Async/runtime facade | Existing async task graphs and code whose I/O already returns futures | Route tasks, synchronization, time and I/O through controlled facilities | A synchronous call inside one poll remains atomic or blocks the executor. |
| B. Request/completion reactor | Existing callbacks, explicit state machines and event-driven storage | Save locals/state and return after submitting operations | Deep sync paths need manual or compiler-generated continuation conversion. |
| C. Suspended Wasm imports | Nested synchronous calls in a supported guest build | Replace the I/O seam; compile guest; govern imports and execution contexts | Host suspension does not automatically run other tasks inside that guest. |
| D. Native fibers/CPS | Nested synchronous native calls; potentially more native libraries | Route effects and synchronization; provide safe continuation lifecycle | Ambient effects, TLS/globals, blocking locks and abrupt kill are difficult. |
| E. Native syscall/VM control | Production binaries, runtimes, threads and more OS behavior | Harness and intercept/control nondeterminism below the application | Greater execution/model complexity; syscall-only control can still miss userspace interleavings. |

B is largely an execution encoding of A for software already written that way. C and D add stack preservation, which is a meaningful integration dimension. E remains on the frontier when retaining a production runtime matters more than narrowly modeling effects.

The coarsest useful schedule depends on the concurrency contract:

| Schedule points | Fidelity and omissions | Cost, state space and replay |
|---|---|---|
| Async waits / Future polls | Good for intentionally cooperative ownership; misses races entirely within one poll | Cheap and small; deterministic executor required. |
| Host imports | Preserves sync stacks and cross-instance interleavings; private computation remains atomic | Moderate boundary cost; operation order makes replay tractable. |
| Synchronization operations | Exposes split critical sections, channel races and lock order | Requires replacing/instrumenting the actual locks, waits and relevant atomics; partial-order reduction helps. |
| Explicit yields | Precisely targets suspected races | Cheap locally but coverage depends on manually maintained placement. |
| Function boundaries | More opportunities, weak relation to concurrency semantics | Inlining/build changes perturb schedules; many redundant states; still misses intrafunction accesses. |
| Compiler instrumentation | Can cover shared accesses, atomics, selected allocations and safepoints | High runtime/toolchain dependence; much larger event streams and proof obligations. |
| Wasm fuel | Bounds guest work and offers reproducible engine-level quanta under pinned configuration | Not native time, not every source instruction, not a guest-thread scheduler; expensive fine quanta. |
| Arbitrary instruction scheduling | Stronger interleaving coverage for the selected machine model | Huge state space; weak-memory behavior still needs an appropriate memory model; replay becomes platform/toolchain-sensitive. |

For `let x = *state.lock(); *state.lock() = x + 1;`, two native threads can lose an update. One non-yielding async poll conceals it. Scheduling before/after the real lock operations can expose it without scheduling every arithmetic instruction. A single blocked Wasm invocation with no independently runnable peer cannot expose it merely because fuel is enabled. [CHESS implementation and scheduling approach](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/chess-osdi2008-chess.pdf)

For single-owner storage components, effect/import boundaries can be a good default. For shared-memory software, the minimum expands to relevant synchronization and multiple continuations. For lock-free algorithms, weak-memory exploration is a separate undertaking: use tools such as [Loom](https://github.com/tokio-rs/loom), and do not attribute that coverage to a cooperative executor. [Shuttle](https://github.com/awslabs/shuttle) is an instructive controlled-scheduling comparator, not a substitute for a crash-persistence model.

Clock correctness also needs care. Freezing virtual time throughout a long computation can hide a lease that expires before use. Model elapsed work at justified checkpoints or use a documented fuel-to-virtual-time policy. Neither reconstructs production CPU timings automatically.

## 4. Does Wasm create a genuinely new architectural point?

**A useful combination, yes; a new continuation primitive, no.** Wasmtime already implements the crucial mechanism. In 48.0.2, a synchronous component import can be backed by `func_wrap_async`; the guest waits synchronously while the host future suspends without blocking the host thread. The engine maintains the guest execution stack on a fiber. This is separate from declaring an asynchronous WIT function. [Component linker implementation](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/crates/wasmtime/src/runtime/component/linker.rs)

A precise minimal architecture is:

1. A host World owns clocks, entropy, pending requests, simulated storage/network state, decisions and trace IDs.
2. Each isolated node has its own Store and invocation future. The future owns or exclusively borrows that Store while suspended.
3. An import validates arguments, copies or otherwise safely owns buffers, registers a request in World, and returns Pending.
4. One deterministic host scheduler chooses the next event and which invocation to poll. Host wakeups make work eligible; they do not independently choose execution order.
5. A completion becomes available only after the scheduler chooses it. Polling resumes the same guest frames and returns from the synchronous import.

The decisive twelve-step example is feasible now. In the executed component probe, node A enters a nested write loop; its import suspends; node B reaches its own import; three of A's seven bytes take effect; the scheduler branches to kill or continue; that prefix may persist; the surviving branch receives a short completion of three; its original stack resumes and submits the remaining four; the loop returns; it calls sync; the host again controls effect and completion; rerunning the same decisions reproduces the trace. Killing is a branch of the execution, not something after which the killed stack resumes.

**Ownership matters more than syntax.** Multiple independent Stores support multiple suspended nodes. Ordinary `call_async(&mut store, ...)` does not let the host make arbitrarily many independently polled calls borrowing one Store. Component entry/reentry has additional restrictions. Do not assume a blocked import can safely reenter that same component to run its other application task. [Store async implementation](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/crates/wasmtime/src/runtime/store/async_.rs)

WASI 0.3 was released in June 2026 and adds async functions, futures and streams. It is relevant to composing async components, but the demonstrated synchronous-import technique does not depend on P3. Current Wasmtime also has concurrent component host APIs and a per-Store concurrency event loop. Those can support richer guest task models; their internal readiness/poll order must be audited or exposed before claiming Adversary controls individual application tasks. P3 is not an automatic port of Tokio, pthreads, or Go's scheduler. [WASI 0.3](https://wasi.dev/releases/wasi-p3), [Wasmtime concurrent component runtime](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/crates/wasmtime/src/runtime/component/concurrent.rs)

Kill should revoke the invocation's ability to issue further effects, cancel/drop the invocation future, run required host cleanup, and release the Store. The tested path does not run guest Rust destructors. Wasmtime's fiber-disposal path resumes host machinery to dispose of it, and its tests explicitly check host-side cleanup on cancellation. That is why host RAII must not be confused with guest process cleanup. Pending effects must be owned outside the disappearing continuation; killing the waiter must not silently erase an already submitted write or remote request. [Fiber disposal](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/crates/wasmtime/src/runtime/fiber.rs), [async cancellation tests](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/tests/all/async_functions.rs)

Buffer ownership must survive every lifetime transition. Copy guest write bytes before suspension, or hold a rigorously valid guest allocation while the operation lives. Never keep a raw pointer into discarded/grown guest memory as a simulated DMA buffer. Stage read results until delivery. Use node-generation IDs so a late completion cannot target a restarted node's reused handle. Separate guest-handle destruction from cancellation of the underlying operation.

Stack disposal is possible; **cheap forkable snapshots are not established**. A suspended continuation includes engine stack state, host futures/closures, Store state, resource tables, globals and memories. Copying linear memory does not clone that whole state. Replay from a known initial state is the conservative implementation. Wasmtime's current record/replay support is not a promise that arbitrary embedding futures and World state can be serialized or forked. [Wasmtime Config, including RRConfig and execution controls](https://docs.rs/wasmtime/48.0.2/wasmtime/struct.Config.html)

Record artifact and engine identities; initial state; selected continuation; request IDs and generations; completion results and byte counts; partial effects; persistence choices; virtual clocks; entropy; cancellation outcomes; crashes; and any allocation/fuel decisions exposed by the model. Unmodeled wall-clock polling, host thread races, randomized collections or parallel guest execution can invalidate replay. Fuel is useful for bounding runaway guest computation; wall-clock-driven epochs are unsuitable as unrecorded deterministic scheduling decisions. [Wasmtime determinism guidance](https://docs.wasmtime.dev/examples-deterministic-wasm-execution.html)

Native alternatives are substantial prior art. The executed corosensei probe preserves the same kind of nested sync code. Its ordinary Drop unwinds the native coroutine and runs destructors; forcing a reset has safety restrictions and is not a general process-kill primitive. TLS, statics, allocators and unmanaged native calls also remain shared unless separately controlled. A subprocess restores a much stronger kill boundary, at the cost of moving toward E. [corosensei](https://github.com/amanieu/corosensei)

Boost.Fiber, user-space actors and compiler CPS transformations establish that continuations do not require Wasm. Emscripten Asyncify can transform call stacks around asynchronous operations, but indirect calls, transformed-call reachability and reentry become part of the maintenance burden. More directly, SimGrid/Sthread already explores native multithreaded programs through interception and controlled execution. Any novelty claim must confront it. [Boost.Fiber](https://www.boost.org/library/latest/fiber/), [Asyncify](https://emscripten.org/docs/porting/asyncify.html), [Sthread paper](https://arxiv.org/abs/2002.06223), [SimGrid model checking](https://simgrid.org/doc/latest/Tutorial_Model-checking.html)

Wasm's strongest contribution here is the combination of stack suspension, enforced imports, independent linear memories and abrupt guest abandonment. Rust traits offer the seam but not enforced ambient-effect exclusion. Dynamic libraries/static linking preserve native execution but share process resources. Containers preserve a real runtime but do not by themselves give deterministic scheduling. None of portable artifacts, ABI stability, reset, scheduling or isolation is uniquely possible with Wasm; the question is their combined cost for the intended target.

FrostDB is relevant prior art for the broader Wasm-DST idea. Its checked-in setup uses a custom Go runtime, a seeded wazero execution and a virtual filesystem; its instructions require the modified Go toolchain and faketime build. That supports feasibility while also demonstrating that Wasm does not eliminate language-runtime work. It is not the same as proving transparent Wasmtime import suspension preserves Go goroutine scheduling. [FrostDB DST instructions](https://pkg.go.dev/github.com/polarsignals/frostdb/dst), [runtime implementation](https://raw.githubusercontent.com/polarsignals/frostdb/main/dst/runtime/run.go)

## 5. What production code becomes newly testable because of it?

“Newly testable” means under this particular deterministic composition and crash mechanism, not that the project had no prior simulation or fault tests.

The inspected sample shows that synchronous continuation problems are common in serious infrastructure, including beneath async APIs. Seven of the nine systems below have important nested or blocking storage paths; FoundationDB and TigerBeetle already express the relevant continuation structure directly. This deliberately selected sample does not establish a population percentage. redb provides the cleanest isolated synchronous seam; several others depend on workers or shared memory that suspension alone does not preserve.

| System | Actual important path | Classification and implication |
|---|---|---|
| redb 4.1.0 | `WriteTransaction::commit` → `commit_inner` → `durable_commit` → page-manager commit → cached-file/backend write and sync | Deep synchronous storage path behind a small explicit backend. Strong C/D candidate for a single owner. |
| Databend Meta / OpenRaft | Raft storage append/vote operations → raft-log → queued chunked-wal flush → native worker batching, writes, sync, callback | Async outer service with correctness-sensitive blocking worker underneath. More difficult than its API suggests. |
| Pebble | `commitPipeline.Commit` → preparation/WAL → apply/publish/sync coordination; log writer flush goroutine → write/sync | Clean VFS but multiple continuations, atomic publication queues and background workers. |
| RocksDB | `DBImpl::WriteImpl`/write grouping → `WriteGroupToWAL` → `WriteToWAL` → log writer; sync and publication coordination | Synchronous calls within native thread/group-commit machinery; C++ stack preservation alone is insufficient. |
| SQLite WAL | Commit → pager → `sqlite3WalFrames` → `walRestartLog`/frame writes; checkpoint → `walCheckpoint` | Deep synchronous VFS plus shared WAL-index and multiple connections. Hostile to one-continuation/private-memory assumptions. |
| etcd | Raft Ready loop → storage Save → WAL Save/encode → encoder flush/fdatasync or segment rotation | Synchronous durable path inside a concurrent Go service; full service also includes transport and mmap-backed state. |
| FoundationDB | Flow actors such as disk-queue `pushAndCommit`, async file writes and queued sync | Compiler-generated continuations already part of production. C is not solving a missing stack seam here. |
| TigerBeetle | Sector-write submission, retained completion structs, short-I/O continuation/callbacks | Production already fits B; preserving the same callbacks is preferable to inventing new stacks. |
| PostgreSQL | `RecordTransactionCommit` → `XLogFlush` → shared WAL coordination / `XLogWrite` / fsync | Synchronous path within shared memory, process coordination and signals. Strong E candidate. |

Implementation references: [redb transactions](https://raw.githubusercontent.com/cberner/redb/v4.1.0/src/transactions.rs), [Databend Raft storage](https://raw.githubusercontent.com/databendlabs/databend-meta/main/crates/server/raft-log/src/impl_raft_log_storage.rs), [Pebble commit pipeline](https://raw.githubusercontent.com/cockroachdb/pebble/master/commit.go), [Pebble log writer](https://raw.githubusercontent.com/cockroachdb/pebble/master/record/log_writer.go), [RocksDB write implementation](https://raw.githubusercontent.com/facebook/rocksdb/main/db/db_impl/db_impl_write.cc), [SQLite WAL](https://raw.githubusercontent.com/sqlite/sqlite/version-3.51.2/src/wal.c), [etcd WAL](https://raw.githubusercontent.com/etcd-io/etcd/main/server/storage/wal/wal.go), [FoundationDB disk queue](https://raw.githubusercontent.com/apple/foundationdb/7.4.7/fdbserver/DiskQueue.actor.cpp), [TigerBeetle storage](https://raw.githubusercontent.com/tigerbeetle/tigerbeetle/main/src/storage.zig), [PostgreSQL WAL](https://raw.githubusercontent.com/postgres/postgres/REL_18_STABLE/src/backend/access/transam/xlog.c).

The following three cases are the decisive integration comparison. Call chains identify logical source layers, not measured optimized stack depths.

**Favorable distributed case: OpenRaft control flow, using Databend Meta as the reality check.** Preserve OpenRaft term/vote/log rules, actual Raft storage glue, raft-log encoding/rotation, chunked-wal batching and its flush callback ordering. The outer `save_vote` operation waits for durable flush completion; append also uses the log/flush machinery. The underlying chunked-wal worker is started with a native thread, receives work, collects batches, performs vectored writes, syncs files and invokes callbacks. Replacing all of this with one abstract `durable_append` removes precisely the acknowledgment/durability pipeline worth testing. [chunked-wal 0.2.0 worker](https://raw.githubusercontent.com/drmingdrmer/chunked-wal/v0.2.0/src/wal/flush_worker.rs)

- A is the natural outer-service fit. It must also control worker queue, waits and I/O. Replacing just Tokio leaves native work outside control.
- B fits Raft events but would require converting the synchronous storage worker and retaining its intermediate state explicitly.
- C can preserve a worker's synchronous stack if its required dependencies compile. A single blocked guest does not automatically let the Raft caller or another worker in the same guest run. Splitting them into components is legitimate only if production ownership/message boundaries already support that split; otherwise it is an architectural rewrite.
- D can preserve native worker code while replacing spawn/wait/lock/I/O operations with controlled equivalents, subject to native isolation and kill constraints.
- E preserves the current thread/runtime arrangement with less source modification, though targeted exploration and semantic explanation need additional instrumentation.

Expected conditional code is the runtime/network/storage/clock/entropy integration, not consensus logic. Full-service Wasm cannot simply inherit native Tokio networking, OS-thread workers, platform file locking, TLS/native crypto and transport assumptions. Some portable implementations may remain; each requires an audited build, not a blanket claim that TLS or all of tonic must disappear. No full Databend port or LOC measurement was performed. **The native flush worker is a demonstrated obstacle, not hypothetical future work.**

**Clean synchronous seam: redb.** Preserve the transaction implementation, B-tree/page management, checksums, commit-slot updates, cached writes and recovery logic. `StorageBackend` has length/read/write/resize/sync operations; source inspection puts several synchronous layers between transaction commit and the backend. That is exactly where a top-level reactor would otherwise need continuation conversion. [StorageBackend](https://raw.githubusercontent.com/cberner/redb/v4.1.0/src/db.rs), [page manager](https://raw.githubusercontent.com/cberner/redb/v4.1.0/src/tree_store/page_store/page_manager.rs), [cached-file implementation](https://raw.githubusercontent.com/cberner/redb/v4.1.0/src/tree_store/page_store/cached_file.rs)

- A would propagate asynchronous behavior through synchronous engine calls, or put the entire operation in a worker. A worker treated atomically fails to expose intermediate effects; a controlled fiber is then D in substance.
- B would save state across that call chain manually or through a transformation. It is an unattractive port for this engine.
- C actually ran without changing the crate. D should preserve the same backend calls natively; its mechanism was executed, but a native redb-fiber port was not.
- E preserves native file backend and reader-thread behavior better; C offers finer, cheaper-to-describe synthetic persistence choices, contingent on a qualified model.

The tested build bypasses native file handles, file locking and platform FileExt/fsync implementations. It uses wasm32 std/allocator/locking behavior and one caller. A single-writer API does not mean production has no concurrent readers. Those omitted readers and native memory behavior must remain explicit coverage exclusions until tested with a supported concurrency model.

**Hostile case: SQLite WAL.** Preserve `wal.c`, pager commit, checkpoint/recovery, native locking/shared-memory interactions, and multiple independent database connections. A VFS supplies synchronous I/O and shared-memory/lock hooks; the hard part is preserving the meaning and interleavings of those hooks and direct accesses to mapped WAL-index state.

- A/B require broad async/manual continuation conversion if used alone.
- C can preserve one connection's synchronous stack. Separate private-memory components do not automatically share SQLite's directly addressed WAL-index memory. One component with one blocked invocation cannot reproduce competing connections. Shared memories or a modeled synchronization/runtime substrate would be additional architecture, not a small adapter.
- D can plausibly preserve the code with a native shared-memory VFS and controlled connection fibers/locks, provided every necessary switch point is available.
- E naturally preserves connections, mappings, mutexes and process behavior; syscall-only scheduling still needs evidence that its switch points include the required window.

Compiling SQLite with omitted shared memory or exclusive/single-connection behavior may produce a functioning database and entirely remove the bug under study. That is failure of this integration experiment, even with a high unchanged-LOC percentage.

## 6. What important code still cannot be preserved?

**Unchanged source does not imply unchanged implementation behavior.** A Wasm target changes pointer width and layout, target cfgs, std/libc paths, allocation and address-space limits, available SIMD/assembly, cryptography backends, initialization, exception/panic behavior, atomics, TLS and threads. mmap, io_uring and native extensions are especially important escape routes. Some differences are harmless for a specific invariant; they must be justified rather than assumed away.

The redb result is strong for its portable commit/recovery algorithm and weak for native OS integration. The sample guest deliberately uses `panic=abort`, custom imports and no native filesystem. It does not test native unwind cleanup or out-of-memory behavior. Similarly, replacing a native crypto implementation with portable code can retain wire semantics while excluding assembly, feature detection and native-library faults.

| Ecosystem | ABI/code portability | Runtime/concurrency portability under this architecture |
|---|---|---|
| Rust | Strong for portable libraries and deliberate trait seams; the redb probe is direct evidence | Tokio workers, OS threads, native locks, target atomics and native dependencies need separate treatment. |
| C/C++ | Many codecs and synchronous libraries can compile; VFS/Env seams are valuable | pthreads, signals, mmap, process-shared state, TLS, exceptions and native assembly are not preserved automatically. |
| Go | Portable packages can target WASI | Ordinary blocking imports can stall the entire module; runtime cooperation is needed to schedule goroutines around I/O. |
| Java/JVM | A WIT ABI says nothing about executing an unchanged JVM server | GC, JIT, monitors, threads, JNI and runtime layout are major parts of the program. Treat whole-server support as a different project. |
| Python | An interpreter plus portable modules may fit a Wasm runtime | Native extensions, event loops, threading, process pools and resource behavior remain substantial exclusions. |

The Go source explicitly documents that blocking WASI calls halt module execution, while nonblocking I/O allows its net poller to schedule goroutines. Host-async implementation of an otherwise blocking guest import does not perform that Go-runtime handoff. This is the clearest example of solving stack suspension while losing the original execution topology. [Go 1.26 WASI syscall implementation](https://raw.githubusercontent.com/golang/go/go1.26.0/src/syscall/fs_wasip1.go)

Language-neutral **artifacts and effect protocols** are credible. Language-neutral preservation of runtime behavior is not presently a defensible product promise. Start with Rust and a constrained C library case only if an actual target requires it.

## 7. Where does this architecture beat Commonware/Turmoil/MadSim?

There is one concrete mechanism advantage: **execute an unchanged synchronous engine call, pause it inside a storage operation, run another isolated node, then kill the first guest without executing its application cleanup.** The redb and component probes establish the pieces. They do not establish superiority over an equally engineered native-fiber backend or a full integration in any competitor.

| Test | Commonware | Turmoil | MadSim | Does Wasm supply the difference? |
|---|---|---|---|---|
| Pause unchanged sync redb halfway through a backend call, while another node progresses | Async Blob/runtime interfaces do not themselves retain a blocked synchronous Rust stack. A fiber/worker bridge could. | Current filesystem support includes sync APIs, but its IoLatency documentation says synchronous operations complete immediately because they cannot sleep in virtual time. A fiber extension could change this. | Async task/filesystem simulation does not itself suspend an arbitrary nested synchronous stack. A worker/fiber bridge could. | Yes for the demonstrated implementation; stackful native alternatives can also do it. |
| Enforce no undeclared native network/filesystem access | Requires discipline, auditing or external containment around linked native code | Same general native ambient-access issue | Same general native ambient-access issue | Import allowlisting and separate linear memory are a meaningful enforcement advantage. Unsafe host imports remain trusted. |
| Crash after write effect but before completion/persistence | A richer storage model can be implemented behind its interfaces | Its evolving filesystem model must be compared operation by operation | The inspected fs code has a no-op sync and explicit unfinished power-failure/buffering work | The effect model creates the value; Wasm does not define persistence. |
| Guide exploration by pending sync, message and acknowledgment dependencies | Possible in a runtime with semantic events | Possible with instrumentation | Possible with instrumentation | No inherent Wasm advantage. |
| Explain an acknowledgment's missing durable dependency | Possible if events and application assertions are present | Same | Same | No inherent Wasm advantage. |
| Load independently built Rust/C node artifacts and kill/reset each in process | Requires an ABI plus an isolation/execution mechanism | Same | Same | Components provide a useful packaging and containment substrate; library/runtime portability is still required. |

Current references: [Commonware deterministic runtime 2026.9.0](https://docs.rs/commonware-runtime/2026.9.0/commonware_runtime/deterministic/index.html), [Commonware Blob](https://docs.rs/commonware-runtime/2026.9.0/commonware_runtime/trait.Blob.html), [Turmoil filesystem implementation and IoLatency notes](https://raw.githubusercontent.com/tokio-rs/turmoil/main/crates/turmoil-fs/src/lib.rs), [MadSim filesystem implementation](https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/fs.rs).

Do not compare with obsolete versions of Turmoil that had no filesystem support, and do not confuse “has a synchronous API” with “can interleave another simulated actor while this call waits.” Conversely, do not claim competitors cannot add the same model. Native fibers, a Wasmtime bridge or a subprocess backend could be added to an existing runtime.

For an already compatible async Rust service, the strongest default is an existing runtime plus missing semantic storage/exploration features. Adversary needs to demonstrate that its supported synchronous-isolation unit is common and valuable enough to justify another execution backend. Without that, “Commonware with a Wasm backend and a richer crash model” is an accurate architectural description, even if the engineering is useful.

## 8. Where does Antithesis/syscall-level execution dominate?

E dominates source preservation for full Pebble/Go services, RocksDB's native concurrency, PostgreSQL, SQLite shared mappings, and applications whose bugs involve native networking, libc, signals, mmap, io_uring, allocators or runtime behavior. Preserving the real binary avoids many changes that otherwise move the bug below or outside the simulation contract.

It does not follow that E universally dominates exploration. A deterministic VM still needs useful workloads, invariants and a fault model, and it does not automatically enumerate every weak-memory/hardware outcome or reproduce every storage device. Semantic hooks can improve it too. Antithesis's deterministic environment preserves much more production machinery; the simulator can potentially explore persistence and causality choices more directly. The performance and bug-finding tradeoff requires a matched workload, not an assumed orders-of-magnitude advantage. [Antithesis deterministic hypervisor](https://antithesis.com/blog/deterministic_hypervisor/), [environment documentation](https://antithesis.com/docs/configuration/the_antithesis_environment/)

Syscall interception, record/replay and a VM are different points. rr primarily records/replays an actual native execution; it is not automatically a search engine for alternative persistence histories. Hermit seeks reproducible native execution through controlled system interaction, with its own platform/syscall constraints. A userspace kernel supplies a larger compatibility surface but does not become deterministic merely by existing. [rr](https://rr-project.org/), [Hermit](https://github.com/facebookexperimental/hermit), [gVisor architecture](https://gvisor.dev/docs/architecture_guide/intro/)

The SQLite WAL-reset defect is a valuable boundary test precisely because it is a real corruption defect. SQLite documents a stale checkpoint view racing with a WAL reset, eventually skipping data during a later checkpoint; the fix shipped in 3.51.3, and an independent unmodified-code reproducer was reported in August 2026. [SQLite's account](https://sqlite.org/wal.html#walreset)

I compared the affected and fixed `wal.c`: after taking read-lock slot zero, the fix checks WAL salts against the live header before doing checkpoint work. This implicates synchronization and shared metadata, not a requirement for arbitrary instruction stepping. However, the stale-header/reset window must actually contain a controllable point. One cannot infer from the location of the fix that existing VFS hooks alone suffice. [3.51.3 implementation](https://raw.githubusercontent.com/sqlite/sqlite/version-3.51.3/src/wal.c)

Current private-memory, one-continuation Adversary cannot represent that execution. A native fiber/VFS harness may, or may need an extra checkpoint at the relevant shared-state access. A threaded/shared-memory Wasm runtime could in principle represent it, but building one substantially changes the cost and boundary. Antithesis has published a reproduction of this known bug; that is evidence of applicability, not evidence that it originally discovered it or that no higher-level harness could. [Antithesis reproduction](https://antithesis.com/blog/2026/wal-reset-bug/)

If most valuable bugs in the chosen users require keeping exactly this native execution structure, choose E or a hybrid and stop marketing C as the central platform.

## 9. The strongest architecture after this investigation

**Build a semantic simulation engine with an explicit execution-context contract, and at most two initially supported execution frontends. Keep native validation mandatory for correspondence claims.**

```text
actual correctness implementation
  ├─ existing async tasks, with controlled runtime/synchronization
  └─ isolated synchronous owner, in a Wasmtime Store
                  ↓
requests + synchronization events + controlled continuations
                  ↓
one deterministic scheduler and semantic effect engine
                  ↓
qualified storage/network profiles, clocks, entropy, failures
                  ↓
trace reduction + invariants + replay + native counterexample validation
```

Reuse an existing Rust runtime's executor/interface work where compatible. The new work should concentrate on operation lifetimes, persistence profiles, continuation ownership, evidence-rich traces and conformance. Do not simultaneously build a replacement Tokio, custom Go runtime, pthread emulator and generic filesystem.

The synchronous Wasm unit should correspond to a legitimate production ownership boundary: an isolated storage owner or process-like node whose correctness-sensitive internal concurrency is supported or intentionally absent. Do not put each production Future in a separate component to evade Store exclusivity; that changes shared state, locking and ownership. Mixing frontends is safe only when communication crosses actual modeled boundaries and their progress dependencies cannot deadlock the host scheduler.

Keep synchronous and asynchronous guest APIs over one semantic operation core. A synchronous import can submit-and-wait while the host separates effects and completion. An async caller can return a request/future. Thus asynchronous **effects** do not require every application's source-level API to be async.

**Cancellation needs a richer lifecycle than one result enum.** `withdrawn | too-late | unknown` is useful for an attempt to withdraw a particular queued operation. It is insufficient as the entire application/task/process cancellation model.

| Action | Required contract |
|---|---|
| Stop waiting | Detach the waiter; the operation may still run. Define how its eventual result/resources are reaped. |
| Cancel local execution | Terminate/drop a task or continuation under a specified cleanup policy; distinguish this from process death. |
| Cancel a queued operation | `withdrawn` must imply no effect has occurred or can subsequently occur, at a defined decision point. |
| Cancellation too late | Withdrawal cannot be guaranteed; some effect may already have occurred. Preserve knowledge of what is known and what remains uncertain. |
| Abort a connection | Affect endpoint/queue/stream behavior; do not erase bytes already delivered or remote work accepted. |
| Operation already effected | Retain the effect even if completion is suppressed or the waiter disappears. |
| Remote operation may execute | Timeout/connection failure gives uncertainty; require application request IDs/deduplication where correctness needs them. |
| Process dies | No guest-language cleanup; external operations may survive. Machine power loss is a separate event with a separate storage fate. |

Rust Future drop is not a universal cancellation primitive for everything it initiated. Tokio task handle drop detaches the task; a running `spawn_blocking` operation cannot ordinarily be aborted. Multi-step I/O can lose progress when a future is discarded. [Tokio JoinHandle](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html), [spawn_blocking](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html), [select cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html)

Go context cancellation communicates a request and does not wait for work to stop. POSIX writes can be partial, and closing a descriptor in another thread need not immediately terminate an already blocked I/O operation. io_uring cancellation has its own completion/result semantics; the original operation's completion and buffer lifetime still require accounting. These differences must be visible in adapter conformance tests. [Go context](https://pkg.go.dev/context), [write(2)](https://man7.org/linux/man-pages/man2/write.2.html), [close(2)](https://man7.org/linux/man-pages/man2/close.2.html), [io_uring cancellation](https://man7.org/linux/man-pages/man3/io_uring_prep_cancel.3.html)

Represent operation execution, waiter attachment, completion delivery and durability separately. A cancellation acknowledgment is itself an event that can race with effect/completion. Permit APIs to expose uncertainty when production does. Never grant the simulator a universal “undo remote operation” result that native code cannot obtain.

**Exploration is where the higher boundary might earn its cost.** Start with seeded simulation and schedule perturbation. Add bounded preemption/delay exploration around synchronization and effect events, then dependency-aware pruning. PCT can target schedules under its stated bug-depth assumptions; it is not a coverage guarantee for arbitrary disk faults. Coverage guidance should include semantic states, not only code coverage: term changes, reply-without-persistence windows, pending syncs, repeated request IDs and recovery branches. Protocol-aware guidance can prioritize useful combinations without enumerating every packet or instruction.

The PCT paper makes its probability argument over a specified scheduling model and bug depth; replacing that model with custom persistence events requires a new justification. [PCT paper](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/paper-83.pdf)

Partial-order reduction can avoid swapping independent operations, but “independent” must include shared file ranges, namespace state, clocks, resource limits, cancellation and task causality. A simplistic rule that two different requests commute can prune the failing execution. A model-checker integration needs an abstract state projection and validated equivalence; hashing heap bytes is neither a stable nor necessarily complete state abstraction.

These techniques are available to instrumented low-level systems too. The plausible advantage is that semantic events are cheaper and more explicit at this boundary. Establish that with a matched bug corpus and CPU budget, not novelty rhetoric. TigerBeetle's recent protocol-aware DST work is an especially relevant comparator. [Protocol-aware DST](https://tigerbeetle.com/blog/2026-08-20-protocol-aware-dst/)

**Debugging can be materially better, but explanations require evidence.** A platform can report: request R wrote bytes; completion C returned; acknowledgment A was sent; no durability event covering those bytes preceded A; after power loss, recovery omitted R. To conclude this is a bug it also needs the application's acknowledgment contract and a recovery/linearizability oracle. The simulator cannot infer transaction identity or causality from file offsets alone. Record provenance and distinguish an observed missing dependency from a proven root cause.

Maintenance should be budgeted around conformance profiles, dependency drift and customer integration, not just the first Wasmtime linker. A constrained Rust SDK, one synchronous backend, one or two storage profiles and good traces are plausibly sustainable by a small specialist team. Multiple language runtimes, arbitrary native threading and a POSIX compatibility layer are not a reasonable small-team scope. The biggest risks are guest/runtime concurrency integration and native-adapter qualification; Wasmtime version pinning and ABI plumbing are secondary until they expose those larger issues.

As a planning budget, not an empirical staffing fact, cap the architecture decision phase at roughly 8–12 engineer-weeks across the experiments below. If that effort still produces only a toy app and exclusions for the real storage worker, the scope should contract. The likely first users are sophisticated Rust infrastructure teams willing to own a narrow platform contract. The evidence does not support thousands of ordinary microservices as the initial market.

## 10. The minimum prototype to build next

The mechanism question has been partially answered. The next work is a controlled comparison, not a general SDK. Predeclare these budgets and gates before running it; the numeric thresholds below are proposed investment limits, not measured industry constants. Preserve a baseline and record engineer-hours, changed production lines, new conditional branches, dependencies replaced, target invariants, throughput, peak memory, replay failures and distinct real defects.

**Experiment A — four-way continuation comparison.** Use one nested synchronous implementation with a real encoding/checksum routine, partial-write loop, sync and acknowledgment, plus a split-lock lost-update case. Implement B as an explicit reactor, A as async conversion, C as suspended imports, and D as native fibers. Give all four the same World model and decisions; verify equivalent event histories before timing. The existing probes are feasibility inputs, not the completed benchmark.

- Measure changed production functions/lines separately from wrappers; count changed semantics, not only LOC. Record which locks/locals/drop behavior survive.
- Run at least 10,000 decisions-bearing schedules and replay each failing/selected trace 100 times. Require zero unexplained replay divergences within a pinned build; test a second supported host separately, without promising bitwise equality across engines.
- Enumerate all distinct crash/effect/completion positions in the small workload and require all four implementations to expose the declared positions or explicitly record a missing one.
- Measure cold initialization separately from steady-state transitions, runs/second and memory per suspended context. Use 1, 10, 100 and 1,000 contexts rather than assuming fiber stack reservation equals committed memory.
- C must retain the synchronous correctness function without async/CPS edits. Failure kills H2 for that target. If D plus robust isolation gives the same contract at lower integration/maintenance cost, choose D/H1. If C is more than 10× slower than the best alternative under identical useful schedules and yields no compensating containment or coverage benefit, remove it from the default path.

**Experiment B — redb beyond the proof.** Pin redb 4.1.0 initially to reproduce this result, then test the current selected release without silently changing both engine and workload. Run the same transaction/recovery workload against native files, a native controlled backend/fiber, and C. Preserve transaction/page/recovery code. Add multiple transactions, page reuse, compaction/resize, savepoints if supported, corruption checks and concurrent-reader scenarios only where the execution model truly preserves them.

- Budget two engineer-weeks for an honest single-owner integration and first qualified profile. Allow wrapper/adapter changes; require zero semantic rewrites of the commit/recovery algorithm. Count native file-backend exclusions explicitly.
- Assemble at least five historical storage regressions when available; otherwise label injected faults as mutants and keep them separate. Include omitted sync, premature durability acknowledgment, truncated record/page, stale commit slot and resize/metadata ordering. Do not invent a historical bug count.
- For each in-scope bug, require reproduction under a declared search budget, successful replay, and disappearance with the fix. Report every miss. An oracle accepting any recovery error as an “old state” is disallowed.
- Run at least 1,000 native storage litmus traces per declared profile and require zero unexplained outcomes outside the model. This is an initial gate, not proof of device equivalence. If qualifying the intended semantics requires reimplementing the filesystem, reject the profile or switch to native execution.
- Require recovery of every acknowledged durable transaction in at least 10,000 selected crash schedules. A passing count alone is not sufficient: demonstrate that deliberate acknowledgment-before-sync mutants fail.

**Experiment C — a real favorable distributed Rust service.** Use Databend Meta's actual OpenRaft storage path, initially excluding SQL/query machinery and external control planes, while preserving Raft logic, raft-log format, rotation/purge and chunked-wal worker batching/callback ordering. Use at least three nodes, real serialization, a client-history oracle and durable-state recovery.

- Four engineer-weeks maximum to produce a scoped integration, with no rewrite of consensus/storage algorithms. Set a provisional cap of 1,000 production integration lines outside those algorithms; separately count new adapter/harness code. Exceeding the cap is evidence to narrow scope, not permission to hide code in a simulator fork.
- Inventory thread creation, channel waits, locks, timers, file calls, transport and native dependencies. Any correctness-sensitive escape remaining uncontrolled makes the corresponding coverage claim fail.
- Inject lost replies, retries with the same request ID, delayed flush callbacks, partial writes, crashes before/after vote/log persistence, restart during snapshot/rotation/purge, election timeouts and cancellation. Target both agreement and no loss of acknowledged writes.
- Compare A through Commonware, Turmoil and MadSim with the same useful scenarios. Reuse whichever can retain the worker behavior most cheaply. Also run actual native processes with controlled network faults, kill/restart and a disk-fault harness; SIGKILL alone is not power loss.
- Require at least one concrete preservation/coverage advantage for C that the native-runtime integration cannot obtain at comparable effort. If competitors find the same defects with less than half the integration effort and C adds no meaningful containment value, stop building a separate platform.
- Treat “it compiles after replacing the real WAL worker with an in-memory model” as failure of this experiment.

**Experiment D — SQLite WAL-reset boundary.** Pin 3.51.2 and 3.51.3. Reproduce the published corruption scenario with real competing connections, then test the identical workload and relevant implementation against each boundary.

- First run unmodified native code with the public reproducer and record its required concurrency. Use a known-result native harness as the reference before comparing simulators.
- B/A wrappers around whole SQLite calls predictably serialize away the window unless transformed more deeply. Measure this instead of claiming success because both calls complete.
- For D, use shared native WAL-index memory and schedule at existing VFS/synchronization hooks. Determine whether a hook occurs after the stale view is established and before the decisive shared-state check. If not, add one explicit diagnostic checkpoint and report that the default hook set missed the bug.
- For C, attempt only the declared supported shared-state/concurrency configuration. Private memories or exclusive mode are an explicit failure to represent the bug, not a valid passing integration. Do not build a JVM-scale threading subsystem just to rescue the test.
- For E, distinguish native unmodified reproduction, syscall-scheduled reproduction and VM reproduction; they are not interchangeable evidence.
- For any claimed capable configuration, require the affected version to fail on the recorded schedule 100/100 times and the fixed version to pass 100/100. If only a source-level checkpoint works, record precisely which default scheduling unit hid it.

D need not make SQLite an intended customer. It must make the architecture's excluded behavior concrete. It can falsify a broad “important bugs only happen at I/O” claim while leaving a well-scoped redb/storage-owner product viable.

## 11. Observations that should kill the architecture

Eight qualitative rejection criteria were written before this investigation and are included with the probes. Their current status is:

| Assumption / rejection condition | Evidence for it | Evidence against it or missing | Falsifier and required response |
|---|---|---|---|
| Sync preservation does not require broad async/CPS rewriting | Executed unmodified redb engine; component mechanism | Full service/worker context not preserved | A/B require algorithm rewrites for clean seams: abandon the claimed integration advantage. |
| The host can schedule suspended Wasm calls deterministically | Two Stores and selected traces replayed; engine implementation supports it | P3/guest-task scheduling and cross-version replay untested | Uncontrollable execution order remains in required configurations: remove that backend/configuration. |
| The Wasm program retains the behavior responsible for target bugs | Real transaction/recovery algorithm survived | Native file backend, readers, Go runtime, SQLite sharing and workers can disappear | Historical failures cease to exist only because dependencies/concurrency were removed: choose D/E or narrow claims. |
| Wasm has enough advantage over native fibers | Enforced imports, private memory, guest kill without guest Drop | D already preserves synchronous stacks; economic comparison absent | Equivalent robust native solution is cheaper: H1/D wins; stop making Wasm mandatory. |
| Chosen scheduling units expose consequential races | Effect boundaries expose durability/recovery windows | Split-lock and SQLite examples defeat broad atomic-turn assumptions | In-scope safety defects require unsupported interleavings: revise units or reject that target. |
| Storage correspondence can be qualified economically | Small profiles and litmus/crash tests are plausible | Current executed byte-vector model is not qualified | Repeated unexplained native outcomes or filesystem-scale emulation required: abandon the model/profile. |
| Existing runtimes do not already provide the same value more cheaply | Sync suspension plus enforced isolation is a concrete combination | Semantic models/debugging can be added to competitors | Matched integration/bug corpus shows cheaper equivalence: contribute features to an existing runtime. |
| Remaining runtime/OS work stays bounded | Single-owner redb proof is small | Native worker/shared-memory/runtime portability can expand scope radically | To preserve the chosen customers, the project becomes a userspace kernel/runtime emulator: prefer E. |

The distinction between narrowing and evasion must be explicit. Dropping full SQLite/JVM/Go support is a reasonable scope change only if the remaining user segment is independently valuable. Reclassifying every failed real integration as “not the intended use case” while retaining a general-platform pitch is not.

The main missing evidence is **not whether Wasmtime can suspend a stack**. It is whether enough valuable production bugs survive the combined build, execution-context and effect substitutions, and whether this produces more useful bug discoveries per integration and maintenance hour than the alternatives.

## 12. The strongest defensible Adversary hypothesis

Adversary is viable if all of the following hold:

```text
valuable invariants depend on a manageable effect/synchronization contract
AND the actual correctness implementation compiles and remains above it
AND required execution contexts remain independently schedulable
AND abstract outcomes can be qualified against native behavior
AND useful executions can be explored and failures replayed economically
AND either
    synchronous-stack preservation plus isolated kill materially lowers integration cost
    OR semantic exploration/explanation outperforms cheaper existing integrations
AND ongoing compatibility costs remain bounded for a narrow user segment
```

The strongest surviving formulation is:

> For systems with explicit ownership boundaries and a tractable set of external effects and synchronization points, a simulator can execute substantial unchanged correctness logic—including nested synchronous storage code—by controlling its continuations and effect histories. Wasmtime is one practical way to preserve those stacks while enforcing isolation and abrupt guest termination. The resulting tests cover a declared abstract execution model, whose correspondence to production must be established independently.

The narrow first product is a Rust infrastructure testing library/engine with an optional synchronous Wasm backend for isolated storage owners, qualified crash profiles, deterministic replay and causal traces. It is not presently a credible universal language-neutral server platform.

A research contribution would require a precise coverage/correspondence argument for the chosen execution units, a validated partial-order/exploration method across effects and continuations, or a substantial real-bug corpus demonstrating an otherwise unavailable cost/coverage point. Merely combining WIT, Wasmtime async imports and a seeded scheduler is established machinery assembled into a tool.

A useful product would require repeated real-project integrations that preserve the relevant workers and libraries, native-validated defects, reliable replay and an integration cost low enough that maintainers continue using it after the initial port. If those experiments fail, the surviving assets are a storage fault model, trace tooling and runtime adapters—not evidence that the original broad architecture was right.
