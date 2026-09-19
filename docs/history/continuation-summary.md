# Historical response

Exported from the research conversation. This records the conclusion at that stage; see the root README for current evidence and later revisions.

## 1. **Does the continuation hypothesis survive?**

**Yes, narrowly. I choose H2: suspended Wasm imports materially expand the synchronous production code that can be preserved. That does not establish a general deterministic-testing platform.**

The strongest correction to the earlier assessment is that **C need not collapse into an async runtime plus a sandbox**. It can preserve synchronous call stacks that the RFC’s explicit reactor would otherwise require converting into continuations.

I tested this rather than relying only on documentation:

| Executed experiment | Result |
|---|---|
| Two Wasmtime components with nested synchronous calls | Suspended imports preserved frames and locals while the host interleaved nodes, short writes, persistence, completions and crashes. Four schedules each produced 101 identical traces. |
| **Unmodified redb 4.1.0** compiled to Wasm | Real commit/recovery code ran against suspended storage imports. Nine selected crash scenarios each replayed ten times successfully. |
| Native Rust fibers using corosensei | Preserved nested synchronous `write_all` calls. Dropping a suspended fiber ran application destructors, demonstrating why ordinary coroutine cancellation is not abrupt process death. |

The redb integration required a **54-line guest wrapper and zero changes to redb**. I verified its 71 packaged files against the registry archive. But it was a **core Wasm module**, while the smaller mechanism test used components. A complete redb component/WIT integration remains untested.

These are feasibility results, **not throughput measurements, filesystem validation, or newly discovered bugs**.

The [full research report](</C:/Users/sattw/Documents/Codex/2026-09-18/you-are-evaluating-a-systems-architecture/outputs/adversary-continuation-review.md>) contains the source-level comparisons, cancellation analysis, scheduling taxonomy and quantitative experiment protocols. The [reproducible probes](</C:/Users/sattw/Documents/Codex/2026-09-18/you-are-evaluating-a-systems-architecture/outputs/continuation-probes.zip>) include source, lockfiles, results and limitations. Research cutoff: **18 September 2026**; recorded investigation time: approximately 41 minutes.

## 2. **Is the effect boundary real?**

**Yes, for declared invariants and execution models. It is not a universal replacement for OS behavior.**

For an isolated process with private volatile memory, crashing during pure computation can often be represented by crashing at an adjacent external-effect boundary. Its private heap and stack disappear either way.

This works only if the simulator preserves the same outstanding operations, messages, clocks and durable state.

| Situation | Effect boundaries usually suffice? |
|---|---|
| Crash inside serialization before bytes escape | Yes |
| Crash between synchronous write and sync | Yes, if both calls can suspend |
| Crash during a write | Yes, with partial-effect and persistence modeling |
| Crash after sync takes effect but before completion returns | Yes |
| Other threads observe modified shared memory | Not without their interleavings |
| Shared mmap survives in another process | Not with private-memory nodes |
| Lock-free publication or weak-memory defect | No |
| Filesystem/kernel/device implementation defect | Only if its relevant behavior is represented accurately |

The included safety classes are substantial: acknowledgment before durability, truncated records, faulty recovery, lost replies, duplicate requests, stale terms, retries and timeout races.

The exclusions also include actual data-loss bugs—not merely operational inconveniences.

**Storage correspondence remains harder than continuation preservation.** Submission/effect/completion/persistence separation is sound. A fixed persistence-unit size is only a model parameter. File length, overlapping writes, rename, directory synchronization, writeback errors and truncation require qualified profiles.

Conformance should test whether native outcomes are admitted by the model, then validate important simulator counterexamples against native execution. A syscall trace alone cannot establish which bytes survive power loss. [CrashMonkey/Ace](https://www.usenix.org/conference/osdi18/presentation/mohan) illustrates why testing the actual filesystem remains necessary.

## 3. **Is the continuation boundary real?**

**Yes. But the missing concept is execution-context topology: which contexts can progress independently, and what state they share.**

Preserving one stack is useful. Preserving the production arrangement of stacks, workers, locks and shared memory is a different problem.

| Design | What it preserves naturally | Principal cost |
|---|---|---|
| **A. Async runtime** | Existing futures and cooperative task graphs | Synchronous paths remain indivisible unless converted or bridged |
| **B. Explicit reactor** | Callbacks and existing event-driven state machines | Deep synchronous code needs saved-state/CPS conversion |
| **C. Suspended Wasm imports** | Nested synchronous guest calls | Guest build differences and internal concurrency |
| **D. Native fibers** | Nested synchronous native calls | Ambient effects, TLS/global state, blocking locks and crash isolation |
| **E. Syscall/VM control** | Production runtimes, threads and more OS behavior | Greater execution machinery and less direct semantic control |

For a single-owner storage component, import/effect boundaries can be sufficient.

For shared-memory software, add synchronization operations and independently schedulable continuations. For example:

```rust
let x = *state.lock();
*state.lock() = x + 1;
```

One atomic Future poll hides the lost-update execution. Scheduling around the actual lock operations can expose it without instruction-level simulation.

Fuel does not solve this automatically. It can interrupt guest execution, but it does not create another runnable guest thread or reproduce native weak memory. [CHESS](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/chess-osdi2008-chess.pdf), [Shuttle](https://github.com/awslabs/shuttle) and [Loom](https://github.com/tokio-rs/loom) address different parts of this problem.

The defensible boundary is therefore:

> **Controlled continuations + controlled effects + the shared-state interleavings required by the target invariants.**

## 4. **Does Wasm create a genuinely new architectural point?**

**It creates a useful combination of existing mechanisms. The continuation primitive itself is not novel.**

Current Wasmtime can implement a synchronous guest import using an asynchronous host function. The guest’s execution waits on an engine-managed fiber while the host regains control. This does not require declaring the guest API asynchronous. [Wasmtime component linker implementation](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/crates/wasmtime/src/runtime/component/linker.rs)

The working architecture is:

1. A host `World` owns requests, storage, clocks and scheduling decisions.
2. Each isolated node has its own Store and invocation future.
3. An import registers a request and suspends.
4. The host chooses effects, other nodes, persistence and completion.
5. Polling the selected invocation resumes its original stack.

The requested twelve-step write/sync scenario is feasible. The executed probe interleaved another node, applied a partial write, considered a crash branch, optionally persisted bytes, returned a short completion, resumed the write loop and suspended again at sync.

Three qualifications matter:

- **A suspended ordinary call holds exclusive access to its Store.** Arbitrarily reentering the same component to run another application task is not a general solution.
- **Host suspension does not schedule other tasks inside the guest.** Separate Stores solve inter-node concurrency, not necessarily intra-node concurrency.
- **Replay is easier than snapshotting.** Copying linear memory does not clone suspended engine stacks, host futures, resource tables and external simulator state.

WASI P3 adds component-level async facilities, but its runtime scheduling must still be brought under deterministic control. It does not automatically preserve Tokio, pthreads or Go scheduling. [WASI 0.3](https://wasi.dev/releases/wasi-p3), [Wasmtime concurrent runtime](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/crates/wasmtime/src/runtime/component/concurrent.rs)

Native fibers can preserve the same synchronous calls. Wasm’s additional value is enforced imports, separate memories and abrupt guest abandonment without application cleanup. Native fibers require more work to achieve that combination safely.

Prior art materially limits novelty: [Sthread/SimGrid](https://arxiv.org/abs/2002.06223), compiler transformations, Asyncify and [FrostDB’s Wasm DST](https://pkg.go.dev/github.com/polarsignals/frostdb/dst) already occupy related territory. FrostDB’s modified Go runtime also demonstrates that Wasm does not eliminate runtime engineering.

## 5. **What production code becomes newly testable because of it?**

The clearest beneficiary is **portable synchronous correctness code behind an explicit I/O seam**.

**redb is a positive result.** Its transaction commit reaches page management, cached writes and synchronization through several synchronous layers. The executed integration retained that engine and its recovery logic. A reactor conversion would be substantially more invasive. [redb transactions](https://raw.githubusercontent.com/cberner/redb/v4.1.0/src/transactions.rs), [StorageBackend](https://raw.githubusercontent.com/cberner/redb/v4.1.0/src/db.rs)

However, this does not mean redb was previously untestable: it already supports alternative storage backends. The added capability is controlled suspension and composition with independently executing, killable nodes.

**Databend Meta is less favorable than its async interfaces suggest.** Its OpenRaft storage path reaches a native chunked-WAL worker that batches writes, synchronizes files and invokes completion callbacks. Those worker interactions are correctness-critical. Replacing them with an atomic “durable append” removes an important part of the target. [Databend storage implementation](https://raw.githubusercontent.com/databendlabs/databend-meta/main/crates/server/raft-log/src/impl_raft_log_storage.rs), [flush worker](https://raw.githubusercontent.com/drmingdrmer/chunked-wal/v0.2.0/src/wal/flush_worker.rs)

**Pebble has a clean VFS but not a simple execution model.** Its commit pipeline, publication queue and WAL flush goroutine must remain independently schedulable. Preserving one Go stack is insufficient. [Commit pipeline](https://raw.githubusercontent.com/cockroachdb/pebble/master/commit.go), [log writer](https://raw.githubusercontent.com/cockroachdb/pebble/master/record/log_writer.go)

The broader source inspection found:

| System | Assessment |
|---|---|
| FoundationDB | Production Flow already supplies continuations; little new benefit from C |
| TigerBeetle | Existing callback/completion structure naturally fits B |
| redb | Strong isolated synchronous target |
| Databend/OpenRaft | Favorable outer control flow; difficult native storage worker |
| Pebble | Valuable synchronous seam, difficult runtime/concurrency preservation |
| etcd | Synchronous WAL inside a concurrent Go service |
| RocksDB | Native threads and group-commit machinery materially affect correctness |
| SQLite WAL | Shared state and competing connections defeat the minimal model |
| PostgreSQL | Shared memory, processes and signals strongly favor native execution |

Seven of these nine inspected systems contain important nested or blocking storage paths. This selected sample demonstrates prevalence, not a population-wide coverage estimate.

## 6. **What important code still cannot be preserved?**

A Wasm build can preserve source while changing the program responsible for the bug.

Relevant changes include:

- Pointer width, layouts and address-space limits.
- Target-specific conditional compilation.
- libc/std, allocators and initialization.
- SIMD, assembly and cryptography implementations.
- Threads, atomics, TLS and synchronization.
- mmap, io_uring, signals and native extensions.
- Panic, exception and cleanup behavior.

The redb probe bypasses its native file backend and uses one caller. It does not preserve native file locking or concurrent-reader behavior.

Go provides a particularly concrete warning: its WASI implementation documents that blocking calls halt the module; nonblocking I/O allows the runtime to schedule goroutines. Making the host implementation asynchronous does not perform that guest-runtime handoff. [Go WASI implementation](https://raw.githubusercontent.com/golang/go/go1.26.0/src/syscall/fs_wasip1.go)

Rust and constrained C/C++ libraries are plausible initial targets. Whole JVM servers, ordinary Python services with native extensions, and arbitrary Go servers entail substantial additional runtime work.

**Language-neutral artifacts are credible. Language-neutral preservation of production execution is not currently a defensible promise.**

## 7. **Where does this architecture beat Commonware/Turmoil/MadSim?**

There is one compelling mechanism example:

> Suspend an unchanged synchronous storage engine inside an operation, run another isolated node, and kill the suspended guest without running its application destructors.

The current async interfaces of Commonware and MadSim do not themselves supply that stack-preserving mechanism. Turmoil now has filesystem support, including synchronous APIs, but its implementation explicitly distinguishes synchronous operations that complete immediately from asynchronous operations with simulated latency. [Commonware runtime](https://docs.rs/commonware-runtime/2026.9.0/commonware_runtime/deterministic/index.html), [Turmoil filesystem](https://raw.githubusercontent.com/tokio-rs/turmoil/main/crates/turmoil-fs/src/lib.rs), [MadSim filesystem](https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/fs.rs)

The other proposed differentiators are weaker:

| Claimed advantage | Assessment |
|---|---|
| Enforced absence of ambient effects | Meaningful Wasm advantage |
| Independent node artifacts | Useful packaging advantage |
| Persistence semantics | Comes from the model; competitors can implement it |
| Semantic exploration | Available to any sufficiently instrumented runtime |
| Causal explanations | Available wherever suitable events and assertions exist |
| Deterministic scheduling | Already central to existing systems |

Competitors could add native fibers or a Wasmtime backend. Therefore, the result establishes **an integration option**, not a durable architectural moat.

For an already compatible async Rust service, an existing runtime remains the default choice. Adversary needs evidence that synchronous-stack preservation plus isolation saves enough integration work to justify its additional backend.

## 8. **Where does Antithesis/syscall-level execution dominate?**

It dominates production-behavior preservation when the relevant bug lives in native runtime, thread, mapping, filesystem or kernel interactions.

SQLite’s WAL-reset defect is the strongest concrete counterexample here. Competing connections and shared WAL-index state can cause corruption. A private-memory, single-continuation component model cannot represent that execution. [SQLite’s account](https://sqlite.org/wal.html#walreset)

I compared the affected and fixed source. The fix checks whether the WAL generation changed after acquiring the relevant lock. This does **not** establish that arbitrary instruction scheduling is required. A native shared-memory harness with appropriate synchronization hooks may suffice—but the necessary race window must actually contain a controlled scheduling point.

That distinction should be tested rather than assumed.

Antithesis has published a reproduction of the known defect. This supports its applicability, while not proving that a higher-level harness could never reproduce it. [Antithesis reproduction](https://antithesis.com/blog/2026/wal-reset-bug/)

Also distinguish the lower-level options:

- **rr:** replay of native executions, not automatically alternative-history exploration.
- **Syscall interception:** preserves native code, but userspace races may require additional control.
- **Userspace kernel:** compatibility machinery, not determinism by itself.
- **Deterministic VM:** preserves substantially more execution behavior, with greater implementation cost.

If preserving the chosen customers requires rebuilding threads, shared mappings and runtime behavior above Wasm, the lower boundary likely becomes simpler overall.

## 9. **The strongest architecture after this investigation**

**A semantic simulation engine with a native async frontend and an optional synchronous Wasm frontend, backed by native validation.**

```text
Production correctness implementation
    ├── controlled async tasks
    └── isolated synchronous owner in Wasmtime
                     ↓
    continuations + synchronization + effects
                     ↓
       deterministic semantic scheduler
                     ↓
 qualified models, replay, invariants, native validation
```

The synchronous component must correspond to a legitimate production ownership boundary. Splitting arbitrary production tasks into separate components changes shared-state semantics and is not a harmless adapter choice.

Keep the effect engine independent of frontend syntax. A synchronous guest can submit-and-wait while the host still separates submission, effects, completion and durability.

Cancellation needs separate state for:

- The execution context.
- The operation.
- Its waiter.
- Completion delivery.
- External effects and persistence.

`withdrawn | too-late | unknown` is useful for withdrawing a particular operation, but insufficient for all cancellation behavior. Stopping a waiter must not erase a delivered request; aborting a connection must not undo remote execution; process death must remain distinct from machine power loss. Tokio, Go contexts, POSIX and io_uring do not share one universal cancellation contract. [Tokio blocking cancellation](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html), [io_uring cancellation](https://man7.org/linux/man-pages/man3/io_uring_prep_cancel.3.html)

The plausible higher-level advantage is semantic exploration: prioritize acknowledgment-before-persistence windows, recovery transitions and conflicting operations, then reduce failures into causal traces.

But neither that exploration nor the resulting explanations inherently requires Wasm. TigerBeetle’s [protocol-aware DST](https://tigerbeetle.com/blog/2026-08-20-protocol-aware-dst/) is important prior art.

## 10. **The minimum prototype to build next**

The stack-suspension question is sufficiently answered to stop prototyping that mechanism alone.

| Experiment | Required comparison | Decisive gate |
|---|---|---|
| **A. Four-way continuation benchmark** | Reactor, async conversion, Wasm suspension, native fibers; identical workload and effect model | Unchanged synchronous correctness function under C; all declared crash points exposed; no unexplained replay divergence |
| **B. Real redb integration** | Native files, native controlled backend/fibers, Wasm | Preserve commit/recovery algorithms; detect deliberate durability defects; qualify a storage profile |
| **C. Databend/OpenRaft storage path** | Adversary versus Commonware/Turmoil/MadSim and actual processes | Preserve real WAL batching, worker progress and callback ordering |
| **D. SQLite WAL-reset** | Affected/fixed versions under each representable boundary | Show exactly which interleaving is available or missing |

Suggested predeclared gates:

- A: 10,000 selected schedules; replay failures and representative traces 100 times; measure throughput and memory separately from compilation.
- B: two engineer-weeks for initial integration; zero semantic rewrites of commit/recovery; native litmus qualification and mutation tests.
- C: four engineer-weeks; provisional cap of 1,000 production integration lines outside correctness algorithms; no replacing the real WAL worker with an atomic model.
- D: any claimed capable configuration must reproduce the affected version on its recorded schedule 100/100 times and pass the fixed version 100/100.

These are proposed investment limits, not empirical constants. The full report specifies workloads, omissions, measurements and failure responses.

## 11. **Observations that should kill the architecture**

The preregistered rejection conditions still apply:

1. **Clean synchronous seams require broad CPS/async rewrites.** Reject the claimed preservation advantage.
2. **Required suspended-call configurations have uncontrollable scheduling.** Reject that backend.
3. **Wasm removes the libraries or concurrency responsible for the target bugs.** Choose native execution or narrow the claim.
4. **Native fibers provide equivalent useful behavior more cheaply.** Make Wasm optional or remove it.
5. **Important in-scope defects occur inside atomic scheduling units.** Refine those units or reject the target.
6. **Storage correspondence cannot be qualified economically.** Stop expanding the abstract filesystem.
7. **Existing runtimes achieve equivalent coverage with materially lower integration effort.** Contribute the missing features there.
8. **Preserving production behavior requires a runtime/OS compatibility project.** Move the boundary downward.

The first two objections were weakened by the executed probes. The others remain unresolved or already invalidate broad claims.

The most important missing evidence is now:

> **Useful real bugs found per integration and maintenance hour, with the relevant production behavior retained.**

Another successful toy workload would add little.

## 12. **The strongest defensible Adversary hypothesis**

> For systems with explicit ownership boundaries and a manageable set of effects and synchronization points, deterministic testing can retain substantial unchanged correctness logic—including nested synchronous storage code—by controlling continuations and effect histories. Wasmtime can supply stack preservation, import enforcement and guest isolation. Production correspondence must be established separately.

This becomes valuable only if:

```text
important behavior survives the build
AND required execution contexts remain schedulable
AND storage/network models admit relevant native outcomes
AND exploration reaches consequential failures efficiently
AND integration and maintenance remain cheaper than alternatives
```

The narrowest credible product is a **Rust infrastructure testing engine with an optional Wasm backend for isolated synchronous storage owners**, qualified crash profiles and strong replay/debugging tools.

A research contribution requires a validated correspondence argument, a better exploration method, or a substantial real-bug corpus demonstrating a new cost/coverage point. Wasmtime imports plus a seeded scheduler are not sufficient novelty.

If the Databend worker integration or matched competitor experiments fail, retain the useful storage models and trace tooling. Abandon the claim that they justify a broad language-neutral execution platform.
