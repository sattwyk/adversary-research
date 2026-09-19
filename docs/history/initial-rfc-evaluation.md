# Historical response

Exported from the research conversation. This records the conclusion at that stage; see the root README for current evidence and later revisions.

**The intermediate boundary is real. The RFC has not yet established that Adversary is a distinct or broadly usable architecture at that boundary.**

I read the entire RFC, including its execution, cancellation, storage, lifecycle, conformance, and evaluation sections. This assessment uses information available through **18 September 2026**. I distinguish proposed experiments and engineering estimates from measured results.

My assessment of your seven possibilities:

| Possibility | Assessment |
|---|---|
| Fundamentally sound | **Yes, conditionally:** for systems whose correctness-sensitive execution already fits an explicit, controlled runtime. |
| Useful only for a narrow class | **Yes initially, and probably permanently relative to ordinary application software.** |
| Already substantially solved | **The central technique is.** FoundationDB, TigerBeetle, Commonware, and others establish this boundary. |
| Based on a false abstraction boundary | **Not categorically.** But several proposed semantics are stronger than native behavior, and the execution boundary excludes more than “native-thread races.” |
| Too expensive to maintain | **Likely if it promises multiple languages and general filesystem/runtime compatibility.** A constrained Rust product is sustainable. |
| Unable to preserve enough production behavior | **For many existing systems, yes.** For deliberately structured infrastructure, no. |
| Capable of becoming useful | **Yes—as a focused runtime, fault-model, exploration, and diagnosis toolkit.** Wasm should earn its place experimentally. |

The strongest objection is not “abstractions inevitably miss bugs.” All these systems abstract something. It is:

> **The RFC makes production correspondence depend on restrictions that can change the concurrency and persistence behavior being tested. It currently proposes tests of the simulator more convincingly than tests of that correspondence.**

---

**What existing systems already establish**

The proposed ladder is misleading. “Application-specific simulator,” “runtime replacement,” and “capability boundary” are overlapping design choices, not successive depths.

FoundationDB’s simulator is application-specific, but its production code also uses runtime interfaces. TigerBeetle retains production protocol and storage logic while substituting environment implementations. Commonware deliberately exposes reusable runtime capabilities. Adversary belongs in this family.

| System | What matters for this RFC |
|---|---|
| **FoundationDB** | Strong evidence that shared implementation plus controlled asynchronous network, disk, time, and scheduling can find consequential bugs. Its success also depends on Flow, disciplined dependencies, workloads, assertions, and years of accumulated fault knowledge. It does not establish cheap retrofit. [Paper](https://www.foundationdb.org/files/fdb-paper.pdf), [testing documentation](https://apple.github.io/foundationdb/testing.html). |
| **TigerBeetle VOPR** | Strong evidence for production protocol/storage-code reuse and precise fault models. Equally important: TigerBeetle added Vortex to exercise release binaries, concrete I/O, clients, and integration behavior that VOPR excludes. That is evidence for complementary testing, not simulator sufficiency. [Vortex technical account](https://tigerbeetle.com/blog/2025-02-13-a-descent-into-the-vortex/), [simulated storage source](https://raw.githubusercontent.com/tigerbeetle/tigerbeetle/main/src/testing/storage.zig). |
| **Commonware** | The closest architectural comparator. Its current deterministic runtime already combines scheduling, clocks, network, storage, replay auditing, and storage fault configuration. “Reusable deterministic capabilities” is substantially occupied territory. [Runtime documentation](https://docs.rs/commonware-runtime/latest/commonware_runtime/deterministic/index.html). |
| **MadSim** | Demonstrates Rust runtime substitution and ecosystem adaptation. But its generic filesystem implementation is not evidence of a complete crash-consistency model: inspected code still contains unfinished power-failure/write-buffer behavior. [Filesystem source](https://github.com/madsim-rs/madsim/blob/main/madsim/src/sim/fs.rs). |
| **Turmoil** | It is outdated to characterize current Turmoil as only a network simulator. Its repository now includes an unstable filesystem substrate with pending/durable state and crash behavior, plus an io_uring integration surface. Adversary must compare against this trajectory, not just historical Turmoil. [Filesystem README](https://raw.githubusercontent.com/tokio-rs/turmoil/main/crates/turmoil-fs/README.md), [repository](https://github.com/tokio-rs/turmoil). |
| **Shuttle** | Relevant to controlled concurrency, randomized scheduling, PCT, and reproducibility. It demonstrates both the utility and integration cost of substituting synchronization/runtime facilities. [Source and documentation](https://github.com/awslabs/shuttle). |
| **Loom** | Addresses a different layer: bounded concurrency and memory-model exploration for instrumented Rust code. Its documented limitations matter; even Loom is not an unrestricted proof of native execution. Adversary does not subsume it. [README](https://raw.githubusercontent.com/tokio-rs/loom/master/README.md). |
| **Hermit** | Demonstrates the lower boundary: deterministic execution of supported native Linux programs through syscall and execution control. Its supported-operation limitations and maintenance-mode status illustrate the cost of growing compatibility. [Repository](https://github.com/facebookexperimental/hermit). |
| **FrostDB’s Wasm DST** | Important positive evidence for pragmatic Wasm execution of substantial Go code. It reported data-loss and duplication discoveries, but required runtime adaptation and retained scheduling/reproducibility limitations. It supports experimentation, not a claim that general Go runtime portability is solved. [Technical account](https://www.polarsignals.com/blog/posts/2024/05/28/mostly-dst-in-go). |
| **Antithesis** | Preserves the surrounding software stack and supports guided exploration, assertions, replay, and debugging. It is not merely a black-box fault injector. Its virtual devices and selected machine environment remain models. [Hypervisor architecture](https://antithesis.com/blog/deterministic_hypervisor/), [test integration](https://antithesis.com/docs/product/writing_tests/). |
| **Jepsen** | Supplies something deterministic execution does not: independently designed workloads, client histories, and consistency judgments. It is a complementary oracle and actual-process testing approach. TigerBeetle’s Jepsen experience exposed an input-generation blind spot across several existing fuzzers. [TigerBeetle’s account](https://tigerbeetle.com/blog/2025-06-06-fuzzer-blind-spots-meet-jepsen/). |

Commonware deserves a precise comparison. Its storage API deliberately provides a stronger abstraction: successful creation/removal has durability guarantees, and startup includes quiescence obligations. Its blob contract already permits partial durability of unsynchronized writes. Adversary’s potential distinction is preserving **file-publication protocols and their mistakes** through separately observable file and namespace synchronization. That is useful, but it is a distinction in storage semantics—not a new architectural category. [Commonware Storage](https://docs.rs/commonware-runtime/latest/commonware_runtime/trait.Storage.html), [Blob](https://docs.rs/commonware-runtime/latest/commonware_runtime/trait.Blob.html).

So: **“Commonware with Wasm” is an approximately correct architectural description, but incomplete as a product description.** Better storage models, conformance, exploration, and explanations could make the product meaningfully different.

---

**1. The boundary captures important bugs, but “important distributed bugs” is not one population**

The useful dividing line is whether the failure can be represented as changes to **observable effects and supported scheduling points while retaining the relevant implementation**.

| Bug class | Coverage under the RFC | Consequence and qualification |
|---|---|---|
| Durability-before-response | Strong | Directly expressible; can detect acknowledged data loss after power failure. |
| WAL encoding, checksums, tail recovery | Strong if actual bytes and recovery code execute | High-value corruption and recovery failures. Testing a replacement log implementation invalidates this claim. |
| Partial writes and reads | Strong within exposed operations | Finds truncation, framing, retry-offset, and recovery mistakes. |
| File-sync/rename/root-sync ordering | Conditional | High-value data-loss coverage, but bounded by the namespace and persistence model. |
| Lost replies | Strong | Exercises uncertainty after execution, retry, and deduplication. |
| Duplicated application requests | Strong with the correct client workload | Must be generated by retries or explicit clients; TCP byte duplication is not the right model. |
| Reconnect races | Partial | Covers stale connection identities and supported task interleavings; misses races inside an atomic poll or unsupported transport runtime. |
| Timeout races | Partial | Covers timer/completion ordering between turns; misses some time changes within a poll. |
| Leader election and replication | Strong for supported execution | Includes safety and recovery bugs, not just liveness. |
| Stale leases and fencing | Conditional | Requires modeled clock rates, pauses, message delays, and execution between checking a lease and acting on it. |
| Clock skew and wall-clock jumps | Strong at explicit clock observations | Does not reproduce all runtime pauses or intra-turn timing. |
| Retry state machines | Strong | Includes duplicate effects, stale responses, and retry-after-restart mistakes. |
| Cancellation | Conditional | Valuable only if native and simulated cancellation/abandonment semantics correspond. |
| Process crash | Strong within lifecycle contract | Must retain machine-level effects that outlive the process. Restart quiescence narrows coverage. |
| Machine crash | Strong for the modeled storage state | Does not test whether the real kernel/device implements that model. |
| Cooperative task races | Strong between modeled polls | Coverage depends on actual polling and synchronization structure. |
| Threaded logical races using locks | Generally excluded inside polls | Can violate safety without unsafe Rust or a low-level data race. |
| Lock-free/native-thread races | Excluded | Can cause corruption, incorrect publication, and lost updates. |
| Weak memory | Excluded | Native atomics and compiler/architecture behavior need separate testing. |
| Kernel readiness/socket behavior | Mostly excluded | Usually availability, sometimes message loss, duplication, or incorrect reconnection at application level. |
| TCP implementation and configuration | Mostly excluded | Transport bugs and resource failures remain; many protocol-level consequences can be approximated. |
| Filesystem implementation bugs | Excluded | Includes genuine loss of synchronized data, not merely theoretical coverage gaps. |
| Runtime/GC behavior | Partial at best | Pause consequences can be approximated; runtime races, finalization, native extensions, and allocator behavior are not preserved. |
| Resource exhaustion | Partial | Logical quotas and bounded buffers are useful, but do not reproduce all native allocation, descriptor, thread, or kernel limits. |
| Application logic unrelated to failures | Depends on workloads and oracles | Determinism does not invent the missing query, transaction, upgrade, or schema-change workload. |

Two counterexamples prevent dismissing the excluded classes as peripheral.

First, PostgreSQL changed behavior after discovering that failed writeback could discard dirty data, while a subsequent `fsync` appeared successful. This is directly relevant to durable database correctness. [PostgreSQL change and rationale](https://www.postgresql.org/message-id/E1gObRL-00026a-N7@gemulon.postgresql.org).

Second, the **2026 SQLite WAL-reset bug** involved concurrent checkpointing and writing, caused corruption, and produced actual service incidents and some lost metadata at Tailscale. SQLite documents the required thread/process concurrency; that execution does not fit Adversary’s current exclusive-storage/cooperative-turn envelope. Antithesis subsequently reported reproducing the known bug. That report demonstrates reproduction, not an independent original discovery. [SQLite’s technical description](https://sqlite.org/wal.html#walreset), [Tailscale’s incident account](https://tailscale.com/blog/sqlite-wal-reset-bug), [Antithesis reproduction](https://antithesis.com/blog/2026/wal-reset-bug/).

**How much important coverage is excluded?**

There is no defensible universal percentage in the available evidence. A percentage across “distributed-system bugs” would depend mainly on how the dataset was selected.

My architectural estimate is:

- For a single-owner, cooperative database control plane, the boundary can cover a large share of **protocol, persistence-ordering, and recovery** failures.
- For an existing multithreaded storage engine, exclusion of synchronization and shared-file execution can remove a substantial part of the safety surface.
- For ordinary services, much incident risk lies in dependencies, deployment, configuration, resource limits, and runtime behavior outside the proposed envelope.

The excluded classes have **zero direct coverage**, even if their consequences can sometimes be imitated. Injecting a pause can test resilience to a GC pause; it cannot find the GC/runtime defect itself.

Measure coverage against a project’s historical safety incidents and defects. Do not substitute a literature-wide percentage.

---

**2. Production correspondence is the central unresolved obligation**

The relevant condition is approximately:

\[
\alpha(\text{native executions on supported deployments})
\subseteq
\text{executions permitted by the model}
\]

Here, \(\alpha\) retains observations relevant to the claimed property: responses, bytes, ordering, failures, and recovered state.

This is stronger than “both adapters pass the same API tests.”

It also creates two separate obligations:

1. **No important native behavior is accidentally excluded.**
2. **A simulated failure is either concretizable on a supported deployment or clearly identified as a violation of an intentionally weaker contract.**

Otherwise the platform produces false confidence in one direction and false alarms in the other.

The most serious drift risks are these.

**The continuation boundary is stricter than the I/O boundary.**

A library can have excellent I/O abstraction and still be difficult to integrate.

Both Pebble’s VFS and redb’s storage backend contain synchronous interfaces. A call expects a result before returning. Under the RFC, effects and completions cannot occur until `node.step` returns. A synchronous caller cannot wait for that completion while retaining its ordinary call stack.

The available choices are:

- Make the call graph asynchronous.
- Hide a larger operation inside an atomic simulator action.
- Suspend the guest stack at an import and resume it later.

The first can be a major rewrite. The second loses the precise crash/interleaving windows being advertised. The third changes the execution architecture. [Pebble VFS source](https://raw.githubusercontent.com/cockroachdb/pebble/master/vfs/vfs.go), [redb backend interface](https://docs.rs/redb/latest/redb/trait.StorageBackend.html).

This is why “only replace the platform interface” can substantially understate integration cost.

**One poll is not necessarily a valid native atomic unit.**

Consider two tasks doing this without an `.await`:

```rust
let old = *state.lock();
*state.lock() = old + 1;
```

These are two separately protected accesses. A multithreaded native executor can interleave them and lose an update. An atomic simulated poll cannot.

This is safe Rust and ordinary locking. The excluded category is broader than lock-free or unsafe code.

Likewise, an operation can check a lease, perform computation, and respond after the lease expires. Freezing time for the whole poll excludes that execution unless an explicit scheduling/time boundary exists inside it.

The RFC’s one-nanosecond eligibility increment prevents a simulator pathology. It is not a production CPU-time model.

**Submission and cancellation have different opportunities to race.**

In simulation, submission followed by cancellation in the same turn necessarily precedes the effect. In production, a kernel or worker may start the effect before the caller attempts cancellation.

A native adapter can conservatively return “too late.” But the simulator must explore that possibility at a corresponding point; otherwise relevant cancellation behavior is absent.

Other drift sources include immediately ready futures versus forced pending futures, wake coalescing, queue order, select fairness, and cancellation of compound operations. Tokio explicitly distinguishes cancellation-safe operations from operations such as `read_exact` and `write_all` whose partial progress matters. [Tokio `select!` documentation](https://docs.rs/tokio/latest/tokio/macro.select.html).

**Cancellation, abandonment, and connection abortion need separate meanings.**

The RFC’s “cancel or drain before close” rule is clean, but can become impractical. An in-flight connect or write may remain unresolved under a partition. If cancellation is too late and close remains busy, a timeout cannot reclaim the connection promptly.

You need separate operations for:

- Stop waiting for the result.
- Prevent additional local progress where possible.
- Abort a connection.
- Acknowledge that remote effects may already have occurred.

Conflating these produces either leaks or guarantees the native adapter cannot honor.

**Worker pools are not implementation trivia.**

Started blocking work may outlive cancellation and process-level task shutdown assumptions. Tokio’s blocking-task documentation explicitly describes this limitation. [Tokio `spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html).

Moving persistence work into an adapter thread pool can alter ordering, shutdown, buffer lifetime, and failure behavior. Those are correctness concerns.

**Build correspondence is separate from source correspondence.**

Wasm may change:

- Pointer width and integer assumptions.
- Allocator behavior and address reuse.
- SIMD and cryptographic implementations.
- Conditional compilation.
- Available libraries and native extensions.
- Threading support and synchronization implementation.
- Panic, exception, signal, and process termination behavior.

“Same source” is valuable evidence, but insufficient.

**A convincing conformance suite**

I would require four layers.

1. **Executable contract tests:** generated operation sequences with aliases, partial I/O, errors, cancellation, close races, buffer pressure, and resource exhaustion.
2. **Temporal tests:** completion before/after effects, delayed notification, effects after cancellation attempts, overlapping reads/writes, and operations surviving process termination.
3. **Crash litmus tests:** write/sync/rename/unlink/truncate combinations, file versus directory synchronization, failed synchronization, reopen, restart, and machine power loss.
4. **Whole-system differential tests:** the same workloads and property checkers against simulated and native builds, with fault schedules translated into native failpoints where possible.

Qualify results by kernel, filesystem, mount options, adapter version, and relevant device assumptions. Testing a live filesystem followed by `SIGKILL` is not a power-loss test.

Ferrite is particularly relevant: it treats filesystem crash behavior as something to specify and check, rather than an incidental property of a fake filesystem. [Ferrite paper](https://unsat.cs.washington.edu/papers/bornholt-ferrite.pdf).

Conformance can realistically maintain a **restricted contract**. It cannot cheaply establish arbitrary POSIX equivalence.

The maintenance threshold is crossed when each new user requires additional emulation of file locking, mmap, shared directories, readiness semantics, native threads, process signals, or library-specific syscall behavior. At that point the interface remains small syntactically while its behavioral implementation becomes a userspace operating system.

---

**3. Storage is the strongest potential differentiator—and the greatest semantic risk**

The separation of **submission, effect, completion, and persistence** is correct and valuable. It exposes mistakes hidden by synchronous fake storage.

But separation alone does not make the allowed persistence outcomes accurate.

**The successful-sync rule needs an explicit error-state model.**

A model in which:

1. A write completes.
2. A sync fails, persisting only some data.
3. A later sync succeeds and therefore persists all previous writes.

is too strong for some real writeback-error behavior.

The PostgreSQL case shows why: dirty data may already have been discarded. A later successful sync need not recover it.

Adversary must either:

- Model lost writeback state and sticky error obligations.
- Require a shared production policy that treats the affected storage as poisoned.
- Implement stronger guarantees in the native adapter, with the associated buffering/recovery cost.

A test-only adapter must not silently repair the application’s error-handling policy.

**The global namespace prefix is a substantive assumption.**

The RFC’s per-root durable prefix of namespace mutations is convenient for simulation. It is not a generic filesystem theorem.

It can exclude outcomes in which independent namespace changes reach durability differently. Conversely, other unconstrained combinations may be impossible on a particular journaled filesystem.

Ext4’s journaling modes and XFS’s logging mechanisms are concrete implementations with specific ordering rules. Neither should be replaced by “POSIX-like” as a justification for the proposed prefix. [Ext4 journal documentation](https://www.kernel.org/doc/html/latest/filesystems/ext4/journal.html), [XFS delayed logging design](https://docs.kernel.org/filesystems/xfs/xfs-delayed-logging-design.html).

The appropriate choices are:

- Prove the native adapter enforces this stronger namespace contract.
- Qualify the contract for specified deployments.
- Replace the global prefix with explicit dependency constraints.

Adding directory synchronization after every native namespace operation would enforce stronger behavior, but could hide precisely the missing-sync bugs you want to find and change performance.

**The persistence-unit model is useful, not universal.**

A fixed 512-byte unit is not a universally valid power-failure atomicity guarantee. NVMe specifies device-dependent atomic-write limits and boundaries. Filesystem buffering and metadata introduce additional structure above the device. [NVMe specification](https://www.nvmexpress.org/wp-content/uploads/NVM-Express-1_4c-2021.06.28-Ratified.pdf).

A byte-level tear profile is more adversarial, but “more adversarial” is not automatically more correct:

- It can reject software legitimately relying on a qualified atomic-write guarantee.
- It still need not model correlated metadata damage or writeback-error state.
- It can spend search budget on impossible combinations.

Use named profiles with explicit assumptions, not a single slider presented as increasing realism.

**Per-file serialization buys tractability by removing behavior.**

One mutation or sync outstanding per inode through completion delivery simplifies the model considerably. It also removes overlap among writes, syncs, callbacks, and notification handling.

Some applications already serialize their file operations; Pebble’s VFS itself documents restrictions on concurrent operations. Others depend on queue depth and group-commit pipelines. Measure the actual target rather than assuming either case.

The key question is whether the adapter merely represents existing ordering or imposes new ordering.

**Process restart quiescence is a contract, not a law of nature.**

Settling old I/O before restart is defensible for a deliberately designed runtime. It can also eliminate races between old work and newly opened resources.

Commonware makes comparable startup obligations explicit. That is precedent for a useful restricted API—not proof that arbitrary existing processes satisfy it.

**What needs a real filesystem/kernel/device?**

At least:

- Journal and recovery implementation defects.
- Incorrect barrier/flush handling.
- Filesystem-specific allocation and metadata behavior.
- mmap and shared-page-cache interactions.
- Delayed writeback error propagation.
- Device firmware and power-loss behavior.
- Native I/O submission, cancellation, and buffer-lifetime defects.

CrashMonkey/Ace found real crash-consistency bugs in mature filesystems. These are not merely theoretical objections to abstraction. [CrashMonkey/Ace paper](https://www.usenix.org/conference/osdi18/presentation/mohan).

ALICE similarly demonstrates that application crash safety depends on concrete persistence properties that vary across filesystems. [ALICE research](https://research.cs.wisc.edu/adsl/Publications/alice-osdi14.html).

My recommendation is a **two-sided storage program**:

- Use abstract models to test the application’s protocol.
- Use native crash testing to qualify the implementation of the abstraction.

Do not expand the abstract model until it pretends to test the filesystem itself.

---

**4. Ordered byte streams are a defensible network boundary**

For many consensus and RPC bugs, this is the right abstraction.

The important behaviors are:

- Arbitrary fragmentation.
- Finite buffering and backpressure.
- Delay and stalled progress.
- Half-close and reset.
- Connection establishment failure.
- Reconnect and stale connection identity.
- Uncertain delivery when a connection fails.
- Different outcomes in each direction.

TCP provides ordered byte delivery; it does not deliver application-visible duplicate bytes merely because a packet was retransmitted. Lost replies and duplicate application requests belong in the client/RPC model. [TCP specification](https://www.rfc-editor.org/rfc/rfc9293.html).

The omissions have different consequences:

| Omitted behavior | Likely impact |
|---|---|
| Retransmission and congestion control | Primarily latency, throughput, and timeout patterns; many consequences are representable as delay/backpressure. |
| Nagle and delayed ACK | Mostly latency and application interaction; approximate delays may expose timeout assumptions. |
| Kernel socket queues/readiness | Adapter bugs, stalls, close races, and resource behavior; requires native testing. |
| Ephemeral ports/TIME_WAIT | Connection exhaustion and recovery problems, usually availability. |
| DNS/service discovery | Can cause wrong-endpoint behavior, stale routing, and failover errors; should be an optional capability if in scope. |
| Socket options | Can change close, timeout, buffering, and reuse semantics; ignoring them can change correctness-relevant behavior. |
| TCP state-machine defects | Outside the model; kernel/transport testing is required. |

Most omitted TCP mechanics are more often operational or performance concerns than direct data-loss mechanisms. But the application’s reaction to those mechanics can produce safety failures.

The boundary survives if Adversary preserves the real RPC/framing/retry code and offers sufficiently varied delay, failure, buffering, and connection histories. It fails if transport integration is replaced by a cleaner implementation that cannot reproduce the production race.

---

**5. The five approaches occupy a frontier, but C is mostly B with additional enforcement**

| Approach | Retrofit and preservation | Coverage and control | Complexity, throughput, debugging |
|---|---|---|---|
| **A. Application-specific DST** | Expensive initial integration; can preserve extensive production code when designed together. | Excellent targeted fault semantics and invariants; coverage is deliberately selected. | Potentially very efficient and highly explanatory; application team owns accumulated modeling knowledge. |
| **B. Runtime/capability interfaces** | Moderate for compatible code, high for incompatible dependencies/concurrency. | Deterministic supported execution; misses behavior below the interface. | Reusable models and exploration; compatibility and facade maintenance remain. Native debugging can be excellent. |
| **C. Capabilities plus Wasm** | B’s integration cost plus compilation/runtime portability work. | B’s semantic coverage, with stronger import isolation and guest-state separation. | Adds ABI/toolchain work and possible overhead; portable artifacts and bounded guest execution are useful. |
| **D. Syscall interception** | Preserves more native userspace and dependencies, subject to supported syscalls and scheduling control. | More native behavior; deterministic replay requires control beyond syscall return values alone. | High compatibility burden; lower semantic visibility unless instrumentation is added. |
| **E. Deterministic VM/hypervisor** | Usually best preservation of an existing supported software stack. Still needs workloads, environment setup, and properties. | Broad OS/runtime execution and repeatability within the virtual machine model. | Expensive platform engineering; snapshots and guided search can offset low-level execution costs. Semantic debugging remains possible. |

Across the requested dimensions:

- **Language and ecosystem compatibility:** E generally has the strongest position for supported native environments; C’s language portability is conditional.
- **Fault-model precision:** A/B/C can express application-relevant faults directly. E offers deeper implementation coverage but does not automatically know the relevant semantic fault.
- **Determinism and replay:** achievable at all five positions within their controlled envelope.
- **Exploration throughput:** cannot be ranked honestly from abstraction depth alone. Workload size, virtual time, snapshots, instrumentation, and search strategy dominate.
- **Ongoing maintenance:** A concentrates cost in one application; B/C amortize common behavior but inherit compatibility demands; D/E own a much broader execution surface.
- **Debugging:** semantic access favors A/B/C, while preservation of native stacks and whole-system history favors D/E. Neither advantage is exclusive.

**C becomes a distinct frontier point only if Wasm produces a measurable benefit that B cannot provide economically**: enforced isolation across many nodes, portable multi-version artifacts, cheap reliable reset, bounded execution, or stack suspension that materially reduces integration work.

With the RFC’s present custom reactor and Rust SDK, **C largely collapses into B plus a sandbox and component ABI**.

---

**6. What Wasm actually contributes**

| Claimed benefit | Real benefit? | Does it require Wasm? |
|---|---|---|
| Isolation | Yes: separate memories and controlled imports help prevent accidental escape. | No. Processes also isolate; traits and ordinary dynamic libraries do not enforce it comparably. |
| Import control | Yes, especially for auditing time, entropy, and I/O access. | No, but Wasm makes it relatively direct. |
| Deterministic reset | Easier to discard guest state without destructors. | No. Process restart also works; host resources and pending operations still need explicit reset. |
| Portable artifacts | Strong practical benefit across compatible hosts. | Not unique, but significantly better than native ABI portability. |
| ABI stability | WIT gives typed, versionable interfaces. | No. Stability remains a versioning and semantic-compatibility obligation. |
| Language neutrality | Useful ABI substrate. | Does not provide library, runtime, or concurrency portability. |
| Snapshotability | Potentially useful. | Not automatic. Compiled-module serialization is not a snapshot of a running distributed simulation. |
| Scheduler control | Supports bounded execution and resumable host interaction. | Does not automatically expose or control guest language tasks. |

Current technology should not be understated. WASI P3 reached 0.3.0 in June 2026 and 0.3.1 in August, with asynchronous component facilities. Wasmtime 48.0.2 was released on 10 September, superseding the RFC’s 48.0.1 baseline. Those developments reduce some plumbing costs; they do not solve correspondence. [WASI P3 releases](https://wasi.dev/releases/wasi-p3), [Wasmtime 48.0.2](https://github.com/bytecodealliance/wasmtime/releases/tag/v48.0.2).

Wasm execution also needs deterministic imports and explicit treatment of features such as floating-point nondeterminism and execution limits. It is not deterministic merely because it is Wasm. [Wasmtime determinism guidance](https://docs.wasmtime.dev/examples-deterministic-wasm-execution.html).

**There is a stronger Wasm architecture than the RFC currently exploits.**

Wasmtime can implement a host function asynchronously while it remains synchronous from the guest’s perspective. The guest stack suspends when the host future is pending and resumes when it completes.

That could preserve a synchronous storage library’s call graph while giving the host control at an I/O import. It directly addresses the continuation problem described earlier. [Wasmtime `func_wrap_async`](https://docs.rs/wasmtime/latest/wasmtime/component/struct.LinkerInstance.html).

But it requires abandoning the rule that all effects wait for an entire atomic `node.step` to return. It also does not solve guest thread synchronization or arbitrary Go scheduling.

Fuel-based resumable yields could eventually permit additional interruption points. They are not equivalent to controlling Rust futures, nor do they reproduce native weak memory. [Wasmtime asynchronous execution](https://docs.wasmtime.dev/examples-async.html).

**If Wasm disappears, nearly all the principal testing value remains:** models, scheduling, faults, replay, invariant checking, minimization, and semantic explanations.

The alternatives are straightforward:

- **Rust traits/static linking:** lowest-friction starting point; weak enforcement against escapes.
- **Dynamic libraries:** ABI flexibility, but poor isolation of globals, threads, and reset state.
- **Subprocesses:** strong isolation and native runtime preservation; more expensive communication and scheduling control.
- **Containers:** convenient packaging and environmental isolation, not determinism.
- **Custom runtimes:** direct control, with ecosystem maintenance costs.

My default would be native Rust first. My exception would be a successful experiment showing that **resumable Wasm imports preserve important blocking libraries much more cheaply than a Rust facade**.

---

**7. Language neutrality is currently an ABI claim**

| Ecosystem | Realistically preservable code | Principal obstacle |
|---|---|---|
| **Rust** | Substantial pure-Rust state machines, codecs, storage logic, and compatible async code. | Tokio assumptions, blocking workers, native dependencies, atomics, and filesystem behavior. |
| **Go** | Substantial pure-Go code can compile; FrostDB demonstrates useful results. | Goroutine scheduling, runtime time/entropy, `net`/`os` behavior, cgo, and synchronous I/O. |
| **C/C++** | Portable single-threaded libraries and engines with replaceable OS interfaces. | pthreads, shared memory, mmap, signals, native extensions, undefined behavior, and continuation handling. |
| **Java/JVM** | Selected algorithms or code adapted to alternative AOT runtimes. | Preserving HotSpot/JIT/GC/JNI/classloading behavior is a different project. |
| **Python** | Pure Python plus a packaged interpreter and supported dependencies. | Native extensions, asyncio integration, threads, subprocesses, and runtime behavior. |
| **JavaScript/C# and others** | Useful restricted components are plausible. | Each runtime needs its own execution and scheduling argument. |

The Component Model ecosystem now has real Go, C/C++, Python, and other language tooling. That establishes more than hypothetical ABI portability. It still does not establish unchanged execution of their production libraries and concurrency models. [Language support](https://component-model.bytecodealliance.org/language-support.html), [Go components](https://component-model.bytecodealliance.org/language-support/building-a-simple-component/go.html), [componentize-py](https://github.com/bytecodealliance/componentize-py).

For Java, an alternative such as TeaVM is not a transparent way to test the production JVM’s behavior. [TeaVM overview](https://teavm.org/docs/intro/overview.html).

Go’s `testing/synctest` is useful adjacent work: controlled time and quiescence testing reduce some pain, but do not turn arbitrary Go servers into deterministic distributed simulations. [Go’s explanation](https://go.dev/blog/testing-time).

The honest eventual claim is:

> “A common simulation host with separately qualified language frontends.”

Each frontend must qualify libraries, runtime behavior, and concurrency independently. WIT does not amortize most of that work.

---

**8. Cooperative scheduling is enough when the production architecture makes it enough**

There is strong precedent for separating correctness-sensitive control from performance work:

- FoundationDB’s Flow-based execution.
- TigerBeetle’s explicit event-driven organization.
- Commonware’s runtime interfaces.
- Dropbox’s Nucleus design, with a controlled core and work delegated around it. [Dropbox technical account](https://dropbox.tech/infrastructure/-testing-our-new-sync-engine).

A viable rule is:

> Parallel workers compute results from immutable inputs; a controlled owner validates and applies those results.

This can preserve performance while making control logic testable.

But workers frequently do more:

- Mutate caches or shared metadata.
- Allocate sequence numbers.
- Publish partial results.
- Perform writes whose order matters.
- Retain references across cancellation.
- Decide when a durability callback fires.

Once those behaviors are outside the scheduler, the simulator is testing a different concurrency surface.

Therefore the architecture needs an explicit **ownership and worker contract**, not just a list of supported synchronization primitives.

Use Shuttle for suitable higher-level interleavings and Loom for small synchronization mechanisms. Neither is replaced by distributed scheduling. Conversely, they do not replace the distributed failure model.

---

**9. Exploration is a credible opportunity, but semantic visibility is established prior art**

A deterministic execution engine gives reproducibility. It does not by itself give effective search.

| Method | Appropriate role | Main limitation |
|---|---|---|
| Random simulation | Broad, inexpensive initial campaigns. | Rare dependent event sequences remain unlikely. |
| Schedule perturbation | Exercise wake, timer, and completion order. | Wastes effort on equivalent schedules without guidance. |
| PCT | Prioritize schedules containing relatively few critical ordering decisions. | Guarantees depend on the specified scheduling model; they do not cover arbitrary storage and distributed faults automatically. |
| DFS/bounded exploration | Small subsystems, short histories, cancellation and persistence litmus tests. | State explosion makes whole-service exhaustive search unrealistic. |
| Coverage-guided exploration | Retain executions reaching new code or semantic states. | Code coverage alone poorly distinguishes distributed states. |
| Protocol-aware exploration | Target term changes, quorum transitions, snapshot installation, and commit boundaries. | Requires trustworthy application annotations and abstractions. |
| Fault-guided exploration | Concentrate crashes and errors around newly observed vulnerable transitions. | A bad fault grammar creates systematic blind spots. |
| Model-checker integration | Check reduced traces or explore bounded implementations. | State extraction and abstraction soundness remain application work. |

Relevant prior art is extensive:

- **WiDS** reused implementations in a simulation/checking environment.
- **MODIST** explored distributed native systems through interposition.
- **SAMC** used semantic information to reduce distributed-system exploration.
- **FlyMC** addressed scalable distributed model checking.
- **Molly/LDFI** used execution lineage to derive meaningful fault combinations.
- **Mallory** and **Legolas** guide distributed fault injection using feedback or inferred program states.  
  [WiDS](https://static.usenix.org/event/nsdi07/tech/full_papers/liu/liu_html/WiDSChecker.html), [MODIST](https://www.usenix.org/legacy/events/nsdi09/tech/full_papers/yang/yang_html/), [SAMC](https://www.usenix.org/conference/osdi14/technical-sessions/presentation/leesatapornwongsa), [FlyMC](https://ucare.cs.uchicago.edu/pdf/eurosys19-flyMC.pdf), [Molly](https://people.ucsc.edu/~palvaro/molly.pdf), [Mallory](https://arxiv.org/abs/2305.02601), [Legolas](https://www.usenix.org/conference/nsdi24/presentation/wu-haoze).

Adversary’s practical advantage could be a common event vocabulary that makes these techniques inexpensive to implement:

- Identify the first response whose durability dependency is unsatisfied.
- Crash after quorum acknowledgment but before local persistence.
- Explore cancellation immediately before and after irreversible effect.
- Reorder only independent file operations.
- Bias exploration toward new combinations of leader term, durable prefix, and connection epoch.

This is a plausible research and product direction.

It is **not unavailable to Antithesis**. Antithesis supports application assertions, coverage, instrumentation, and guided exploration. Its current Raft report describes safety failures found under network turbulence in several implementations. [Antithesis exploration](https://antithesis.com/docs/introduction/how_antithesis_works/), [2026 Raft results](https://antithesis.com/blog/2026/finding-bugs-in-raft-implementations/).

The differentiator must be measured: less annotation, better bug discovery per unit of compute, or faster diagnosis—not privileged possession of semantics.

Also, fault generation cannot compensate for poor workloads. TigerBeetle’s Jepsen experience is a warning: several fuzzers shared a workload blind spot. More seeds would not necessarily repair that.

---

**10. Debugging could be a substantial advantage**

A useful failure artifact would say:

```text
request 417 accepted
WAL range [8192, 8448) write completed
response “committed” emitted
power loss selected a permitted state without that range
recovery discarded transaction 417
client history violates durability
```

That is substantially better than an unexplained failed assertion and a large syscall trace.

However, the explanation must distinguish:

- No durability barrier was established.
- The model selected an outcome where the data was absent.
- Native execution has reproduced that outcome.
- A stronger synchronization placement prevents the failure.

These are different claims. Data can persist without a sync; lack of a sync alone does not prove that a particular native execution lost it.

Build explanations from an explicit dependency graph, then validate them through counterfactual replays. Do not infer causality from event proximity.

This could justify meaningful integration work for storage teams. But debugging quality needs a user experiment: give engineers equivalent failures from Adversary and a baseline, then measure time to correct diagnosis and repair.

Antithesis already offers replay and the ability to revisit executions with richer diagnostics. Adversary needs better *semantic explanations*, not merely another replay UI. [Antithesis debugging guidance](https://antithesis.com/docs/best_practices/optimizing/).

---

**11. Integration experiments that can actually reject the architecture**

The RFC’s production-intended service is insufficient as the principal validation target. A system designed around the proposed contract can demonstrate usefulness while concealing retrofit and correspondence costs.

Use existing implementations whose structure the experiment is not allowed to simplify.

The following LOC figures are **proposed stop budgets, not estimates derived from completed ports**. Count existing semantic code changed separately from mechanical imports, generated bindings, and new adapters.

**Experiment A: Databend Meta—test the strongest plausible Rust segment**

Preserve:

- The real Raft implementation and integration.
- `crates/server/raft-log`, including codecs, storage operations, and durability callbacks.
- State-machine and snapshot code.
- Actual RPC framing and retry behavior.
- The underlying log implementation and its recovery bytes.

The current log integration explicitly separates appending, flushing, and callback completion. This is an excellent test of whether Adversary preserves a meaningful existing durability pipeline. [Workspace](https://github.com/databendlabs/databend-meta), [Raft log integration](https://github.com/databendlabs/databend-meta/blob/main/crates/server/raft-log/src/impl_raft_log_storage.rs).

Required changes:

- Runtime/task/time/entropy interfaces.
- File and namespace operations.
- Transport construction and connection handling.
- Worker completion control.

Dependencies to audit include Tokio, tonic/h2, TLS/crypto, blocking storage workers, synchronization, and snapshot storage. A simulation-only replacement for the Raft storage trait that omits the actual log engine is a failed experiment.

Faults:

- Acknowledgment before durable vote/log state.
- Lost replies and duplicated client requests.
- Snapshot publication followed by premature log deletion.
- Short writes, torn tails, failed flushes.
- Cancellation during reconnect or shutdown.
- Process crash versus machine crash.

**Stop budget:** six engineer-weeks for the initial native deterministic integration; no more than 3,000 existing non-generated LOC changed. Treat this as a demanding gate for the claimed sweet spot, not a promised schedule.

Baselines:

- Turmoil with the same application code and relevant filesystem model.
- MadSim plus the same custom durability model where needed.
- Commonware where its runtime/storage semantics fit; document any semantic mismatch.
- Three-to-five native processes using real clients, targeted failpoints, network disruption, and genuine machine-crash testing.

**Failure condition:** successful integration requires replacing the log, transport state machine, or concurrency architecture; or production performance depends on overlap the contract forbids.

**Experiment B: Pebble—test whether a natural storage interface is enough**

Preserve:

- WAL writer and reader under `record/`.
- Manifest/version logic.
- Commit pipeline.
- Compaction and table generation.
- Open/recovery code.
- Actual VFS call ordering.

Pebble already has a substantial VFS and a crashable in-memory filesystem. Adversary must beat that existing baseline, not a simplistic mock. [VFS](https://raw.githubusercontent.com/cockroachdb/pebble/master/vfs/vfs.go), [crashable memory filesystem](https://raw.githubusercontent.com/cockroachdb/pebble/master/vfs/mem_fs.go).

Required changes expose two architecture questions:

1. Can synchronous VFS calls suspend without rewriting the engine?
2. Can the relevant Go goroutine execution be controlled without a private, perpetually patched runtime?

Faults:

- WAL sync and commit acknowledgment.
- Manifest replacement and SST deletion.
- Compaction racing with flush/recovery.
- File-length changes and torn writes.
- ENOSPC and writeback failure.
- Crash after error followed by apparently successful synchronization.

**Stop budget:** a two-week feasibility gate for continuation/runtime control; at most 1,500 existing engine LOC changed for the initial port. If this requires an async rewrite, stop and record that result.

Baselines:

- Pebble’s current crashable VFS and tests.
- Native Go concurrency/race testing.
- A native process driver with filesystem crash tests.
- MadSim/Commonware/Turmoil are not direct Go baselines. Porting Pebble to Rust would test a different proposition.

**Failure condition:** the implementation can only run by making large storage operations atomic or replacing production goroutine interactions with a simulator-specific algorithm.

That would falsify the “existing I/O seam implies cheap integration” claim.

**Experiment C: SQLite WAL—test the boundary’s known safety exclusion**

Preserve `wal.c`, `pager.c`, transaction/checkpoint logic, actual database format, and the synchronization protocol around multiple connections.

Run the documented WAL-reset regression, plus independent concurrent checkpoint/write workloads.

Required surfaces include synchronous VFS calls, shared-memory coordination, locking, and multiple execution contexts. These exceed the current RFC.

That is deliberate, but the interpretation must be fair:

- Failure here **does not kill a narrowly scoped cooperative runtime**.
- It **does kill a broad claim that the boundary preserves the important behavior of existing storage engines**.
- Success would require an explicitly extended execution/storage architecture.

Baselines:

- Native SQLite with concurrent connections.
- Its existing VFS/testing machinery.
- The published Antithesis reproduction.
- Rust runtime simulators are not direct substitutes for this native C concurrency test.

**Stop budget:** one week to demonstrate representability or document the architectural obstruction. Do not spend months disguising a new scheduler/shared-memory system as an adapter.

**Common quantitative gates**

Pre-register these before implementation:

- **Preservation:** no second implementation of WAL encoding, recovery, RPC retries, or durability ordering.
- **Historical regressions:** select at least eight relevant historical defects, including four safety/data-loss defects, plus independently planted mutations.
- **Representability:** every claimed in-scope regression must have a constructible legal schedule. Random failure to find one is not enough to diagnose a modeling problem.
- **Replay:** zero divergence across 100 replays of every discovered failure on two qualified hosts, with artifacts and engine versions pinned.
- **Conformance:** no observed native trace outside the claimed model. A counterexample must change the contract or remove the claim.
- **Search:** compare repeated campaigns under equal compute budgets; report discovery distributions and unique defects, not only executions per second.
- **Wasm:** compare the same model and target with native deterministic execution. Use the RFC’s threefold slowdown ceiling as a provisional gate, but also require a demonstrated benefit.
- **Production impact:** provisionally reject changes causing over 5% throughput loss or 10% p99 latency regression on agreed workloads unless the project accepts the tradeoff.
- **Diagnosis:** blinded engineers must identify and repair failures faster, or with fewer incorrect hypotheses, than with baseline artifacts.

These thresholds are proposed decision rules. They are not claims about achievable performance.

---

**12. Maintenance economics and actual users**

The expensive part after a prototype is unlikely to be implementing the initial event queue.

| Component | Ongoing burden |
|---|---|
| Rust runtime facade | Dependency upgrades, runtime behavior, cancellation, synchronization, and integration support. |
| Storage models | Persistent semantic review, profile qualification, error behavior, and misleading counterexamples. |
| Native conformance | Kernel/filesystem/build matrix, crash infrastructure, regression triage. |
| Language SDKs | A separate runtime-portability project for each ecosystem. |
| Wasmtime/WIT | Engine upgrades, bindings, feature changes, deterministic configuration, debugging. |
| Exploration | State representation, search heuristics, workload diversity, minimizing histories. |
| Debugger | Mapping events back to source, causal explanations, stable replay artifacts. |
| Build integration | Feature flags, native dependencies, conditional compilation, release parity. |

The largest traps are **language runtimes, storage semantics, and production adapter qualification**. Wasmtime compatibility is real work, but probably not the dominant cost if Wasm remains optional.

My planning estimate—not an industry benchmark—is that **two to four experienced full-time maintainers** could sustain a useful Rust-focused system with a limited deployment matrix and contributing pilot projects. That team should not promise production-quality frontends for Rust, Go, C++, JVM, and Python simultaneously.

The user segments differ sharply:

| Segment | Adoption case | Likely rejection reason |
|---|---|---|
| Greenfield database/storage systems | Strong: architecture can accommodate controlled effects early; safety defects justify investment. | Existing bespoke simulator or runtime dependency preference. |
| Consensus libraries | Moderate: useful for async integration and persistence obligations. | A pure state-machine harness already covers much of the library; storage/transport bugs belong in downstream applications. |
| Rust distributed infrastructure | Strongest retrofit candidates when dependencies are manageable. | Tokio/native dependency surface and worker concurrency are too extensive. |
| Ordinary microservices | Usually weak. | Most risk lies in business workflows and external services; integration cost exceeds expected benefit. |
| Existing databases | Potentially enormous value, but poor fit for the proposed integration boundary. | Threads, mmap, native extensions, complex storage and mature alternative tests. |
| Language/runtime developers | Weak fit. | The abstraction removes the implementation they need to test. |

For a compatible existing Rust service, plan around **weeks to a few person-months**, including workloads and oracles, rather than “implement a handful of traits.” For a legacy threaded database, the work can become an architectural rewrite.

The realistic initial market may indeed resemble **tens of sophisticated infrastructure projects**. There is no evidence here for thousands of readily addressable applications. A reusable open-source tool for twenty committed projects could still be valuable; its documentation, governance, and economics should reflect that audience.

---

**13. The assumptions that must survive**

A useful dependency tree is:

```text
Adversary is valuable
  IF relevant production correctness code is retained
  AND important native executions fit the model
  AND workloads and oracles expose meaningful failures
  AND integration and maintenance cost are acceptable
  AND (
       exploration is materially better or cheaper
       OR diagnosis is materially better
       OR reusable infrastructure saves substantial bespoke work
      )

Making Wasm mandatory additionally requires:
  Wasm provides a demonstrated benefit
  AND that benefit exceeds portability, performance, and tooling costs.
```

| Assumption | Supporting evidence | Contrary evidence | Falsifier and response |
|---|---|---|---|
| Effect interfaces preserve important correctness code | FDB, TigerBeetle, Commonware. | Their architecture and dependencies were deliberately controlled. | External Rust pilot needs a second log/transport implementation: narrow scope or abandon the adapter claim. |
| Atomic cooperative polls preserve needed concurrency | Successful single-owner/event-driven systems. | Ordinary locked code and runtime workers can interleave within a poll. | Historical safety defect cannot be represented: extend scheduling or exclude that architecture explicitly. |
| Storage model covers supported deployments | Useful abstract crash models and existing simulator practice. | Filesystem variability and writeback-error behavior. | Native crash outcome violates the model: revise semantics before claiming coverage. |
| Adapters remain small | Narrow explicit capabilities. | Synchronous continuation, mmap, locking, runtime and thread-pool dependencies. | Repeated per-target compatibility work dominates: switch to a lower boundary or narrow users. |
| Wasm pays for itself | Isolation, artifacts, resumable imports. | Most simulation functionality exists without it. | Equal bug yield with worse cost and no operational benefit: remove mandatory Wasm. |
| Semantic visibility improves search | SAMC, LDFI, Legolas and related work. | Benefit often depends on application-specific annotations. | Equal-budget campaigns show no improvement: stop claiming exploration differentiation. |
| Better traces improve debugging | Direct durability and request relationships. | Existing platforms already support rich instrumentation/replay. | Blinded diagnosis shows no improvement: simplify debugger ambitions. |
| Small-team maintenance is sustainable | Restricted Rust/runtime scope. | Each language and deployment expands qualification obligations. | Two pilots consume nearly all maintenance capacity: freeze scope before adding frontends. |
| Users will adopt | High cost of infrastructure safety failures. | Mature teams already have simulators or prefer native binary testing. | No external project keeps the integration after a release cycle: reconsider the product. |

---

**14. A stronger architecture**

I would change the project into a **Rust-first deterministic execution and fault-analysis library with optional execution backends**.

Its structure should be:

```text
Unchanged application correctness code
                 |
       qualified runtime interfaces
                 |
   shared event, fault, and property engine
        /               |                \
 native deterministic   optional Wasm     native-process
 execution              execution         validation
```

The important changes:

1. **Make Wasm optional immediately.** Keep the event model independent of Component Model bindings.
2. **Adopt or extend an existing Rust runtime where practical.** Compare an extension of Commonware or Turmoil against a new facade before creating another ecosystem.
3. **Treat continuation support as a first-class design decision.** Prototype resumable Wasm imports separately from the atomic-turn reactor.
4. **Specify a production concurrency envelope.** Single-owner control, allowed workers, shared state, cancellation, and task boundaries must be explicit.
5. **Use multiple qualified storage profiles.** Include writeback errors and synchronization obligations; do not call a single profile “the filesystem.”
6. **Separate aborting local resources from canceling remote effects.**
7. **Build native conformance and crash testing alongside simulation.**
8. **Replay semantic fault constraints against native binaries.** Exact instruction replay is unnecessary for this validation path; preserve causal fault placement where possible.
9. **Invest early in workload diversity and independent oracles.**
10. **Make semantic failure explanation a primary deliverable.**

Other alternatives have narrower advantages:

- **WASI syscall-like execution:** improves library compatibility but inherits broader OS semantics and still does not solve runtime scheduling.
- **Native process plus deterministic I/O proxy:** preserves runtime behavior, but I/O control alone does not make thread execution deterministic.
- **Record/replay plus semantic injection:** attractive for diagnosing production-like failures; record/replay alone does not search alternative executions. [rr](https://rr-project.org/).
- **seccomp/eBPF interception:** useful observation/control mechanisms, not complete determinism architectures.
- **Deterministic userspace kernel:** potentially powerful, but moves maintenance toward D/E.
- **Simulator plus VM:** the strongest practical combination when application-level exploration and native-stack validation are both important.

For an existing broad software stack, Antithesis-style execution may dominate on total adoption cost. For compatible infrastructure developed with controlled effects, a library-based simulator may dominate on transparency, local iteration, targeted faults, and ownership.

Neither universally dominates the other.

A recent example worth watching is **Themis**, an August 2026 preprint exploring whole-machine filesystem checking and replay. Its current evaluation is narrow and explicitly does not establish a general crash-consistency solution. It reinforces that lower-level approaches are evolving too; Adversary should not compare itself against a static or maximally expensive conception of VM testing. [Themis preprint](https://arxiv.org/html/2608.01135v1).

---

1. **What is actually novel here**

   No fundamental boundary has been established as novel. A reusable combination of explicit effect/persistence semantics, qualified native correspondence, semantic exploration, and causal failure explanations could be distinctive. Wasm packaging alone is not.

2. **What is already solved**

   Shared implementation under simulated runtime interfaces; deterministic scheduling and replay; network/storage fault injection; property checking; semantic exploration techniques; and lower-level execution of existing software. Existing systems solve different subsets with different costs.

3. **What assumptions appear correct**

   OS-wide control is unnecessary for many consequential infrastructure bugs. Preserving WAL, framing, retries, recovery, and transaction code is valuable. Ordered byte streams are a useful boundary. Completion and persistence must be distinct. Cooperative execution works for deliberately structured systems.

4. **What assumptions appear wrong**

   A small API implies a small compatibility burden; an I/O seam implies a cheap continuation seam; poll-level control captures all ordinary task races; Wasm implies language neutrality; and one abstract storage model can stand in for production filesystems without substantial qualification.

5. **Most important missing evidence**

   An independently selected production system integrated without replacing its correctness mechanisms, accompanied by native conformance evidence and equal-budget comparisons against current runtime simulators.

6. **Narrowest user segment where this clearly works**

   Rust infrastructure with single-owner control logic, explicit asynchronous effects, controlled dependencies, and a team willing to maintain workloads, invariants, and a defined deployment envelope.

7. **Architecture changes you would make now**

   Native Rust first; optional Wasm; explicit concurrency and continuation contracts; qualified storage profiles; realistic error/cancellation semantics; and mandatory native validation alongside simulation.

8. **Experiments that could kill the project**

   Databend Meta requires a second implementation or major serialization changes; Pebble cannot retain synchronous storage execution; native crash outcomes contradict the model; Wasm contributes no measurable benefit; or existing runtimes deliver equivalent results at substantially lower integration cost.

9. **What would make this a research contribution**

   A demonstrated method for qualifying abstraction-level production correspondence, a materially better semantic exploration algorithm, or validated causal explanations that reduce diagnosis time. Evaluate on preserved real implementations and independent defects.

10. **What would make this a useful product**

   A small, maintained toolkit that several external infrastructure projects keep using across releases because it finds safety bugs or reduces diagnosis cost beyond their existing tests. A language-neutral platform claim is unnecessary—and currently makes that outcome less likely.
