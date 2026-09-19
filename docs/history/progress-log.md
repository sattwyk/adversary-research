# Research progress log

User-visible research updates exported from the prior task. These are historical observations and intentions, not evidence that every planned action completed. Internal reasoning and raw tool/session data are excluded.

## 2026-09-18T12:15:33.000Z — completed

I’ll read the full RFC, then test its claims against implementations and primary research. I’ll focus on production correspondence, whether Wasm adds a distinct architectural advantage, and experiments that could falsify the design.

The RFC already acknowledges most obvious limitations, so repeating them would not test the hypothesis. Two stronger concerns stand out: its scheduling and storage restrictions may remove production executions, and the mandatory Wasm layer needs to demonstrate value beyond a native deterministic runtime. I’m checking both against current code and documented failures.

The current competitive baseline is stronger than “Commonware with Wasm” suggests: Commonware documents torn-write and restart obligations, and Turmoil now has a filesystem simulator. I also found a concrete reason to retain native testing: TigerBeetle’s Vortex caught client batching and connection-lifecycle bugs outside VOPR’s coverage. The useful distinction is becoming which production behaviors each approach preserves, not which simulator controls more knobs.

I’ve found three concrete pressure points in the contract: synchronous storage libraries cannot simply await its completion API; freezing time and effects for an entire task poll hides some lease and cancellation races; and a successful `fsync` after an earlier writeback error needs more careful treatment than the current durability rule provides. These are good falsification targets because they can fail even when the same application source is compiled into both builds.

The strongest test should use three existing implementations: Databend Meta’s replicated storage path, Pebble’s storage/recovery path, and etcd’s Raft/WAL/transport integration. That combination tests a plausible Rust adopter, the synchronous-storage problem, and the language/runtime boundary. I’ll distinguish failures that merely rule out a target from failures that should kill mandatory Wasm or the platform contract itself.

One consequential issue is the continuation boundary: a library may have a clean I/O interface yet still be difficult to port because its callers expect synchronous results. Wasmtime can suspend a guest inside a synchronous host call, which suggests a stronger alternative to the RFC’s atomic-turn model. I’m separating that possibility from benefits Wasm provides merely through packaging and isolation.

The evidence supports the boundary for deliberately structured infrastructure, but it weakens the case for a new language-neutral platform. Commonware and Turmoil now overlap more of the proposal than older comparisons suggest. Conversely, the 2026 SQLite WAL-reset incident is a concrete data-loss example that the RFC’s concurrency and shared-storage restrictions would exclude. I’m using both findings to define experiments that can distinguish a useful narrow product from an overly broad claim.

## 2026-09-18T12:50:17.000Z — failed

I’ll read the goal objective and check what remains beyond the evaluation already delivered.

The new objective focuses on whether controlled suspension can preserve synchronous production call stacks. I’ll treat major async/CPS rewrites, loss of relevant runtime behavior, uncontrollable resume order, and cheaper equivalent coverage in existing runtimes as reasons to reject or narrow the design. I’ll inspect the runtime mechanisms and real call paths, then attempt a minimal executable prototype if the local toolchain supports it.

Wasmtime’s implementation confirms that a host-async import can suspend an ordinary synchronous guest stack. The important limit is that the suspended call retains exclusive access to its store. Separate node stores can be interleaved; this mechanism alone does not let another task run inside the same blocked guest runtime. I’m testing that distinction directly.

The executed probes support a narrower version of the hypothesis. Wasmtime suspended and interleaved two synchronous components with deterministic replay. Unmodified redb 4.1.0 also compiled and ran against suspended storage imports: its commit completed normally, and fresh instances recovered after simulated crashes partway through each of four commit writes. This demonstrates continuation preservation; it does not validate native concurrency or filesystem correspondence.

## 2026-09-18T16:58:51.000Z — completed

The main limitation is now clearer: preserving a synchronous call stack does not preserve the other threads or tasks that normally run alongside it. I’m checking that distinction against the storage engines’ actual commit paths, and packaging the probes so the positive result can be reproduced without overstating it.


The SQLite counterexample sharpens the boundary. Its WAL-reset defect involves shared WAL-index state and competing connections; suspending file calls alone does not represent that execution. The fix also suggests that suitably placed synchronization hooks may expose it without instruction-level scheduling. That is a narrower—and more useful—test than assuming either “Wasm catches it” or “only a VM can catch it.”


The review now separates feasibility from value. Stack preservation works on the tested redb path; the unresolved question is whether it retains enough production concurrency and library behavior to outperform a native runtime integration. The proposed kill tests target that question directly, including Databend’s real WAL worker and the affected/fixed SQLite versions.


The research report and reproducible probes are complete. The tests passed, and I verified that the redb crate was unchanged. The conclusion is conditional H2: Wasm suspension preserves useful synchronous production code, but native concurrency, storage correspondence, and the cost of integration remain decisive—and unproven—advantages for the proposed platform.


## 2026-09-18T17:33:11.000Z — failed

I’ll first map and pin the real Databend durability path before changing it, then test whether its WAL worker can remain intact under controlled scheduling. I’ll record the kill criteria and time spent by phase, and keep any environment limits, unrun comparisons, and lost production behavior explicit.


## 2026-09-18T17:35:50.000Z — failed

The current target has changed since the earlier probe: Databend Meta now pins raft-log 0.4.6, which uses chunked-wal 0.2.3. I’m using that current path. Linux is available through WSL, so I can test the Unix worker and real filesystem directly rather than treating Windows build limitations as an architectural failure.


## 2026-09-18T17:43:37.000Z — failed

Yes. I’ll keep the experiment focused on consensus, durability, recovery, and deterministic scheduling, and leave cybersecurity testing and security-focused features out. The production WAL worker and correctness checks remain the core of the university project.


## 2026-09-19T00:44:39.000Z — failed

I’ll continue the university experiment on durability, recovery, and deterministic scheduling. First I’ll check the build and saved evidence so the next tests preserve the real WAL worker and its callback ordering.


I’ve read the updated objective in full; it keeps the same experiment and preservation rules. The baseline build failed because Databend’s configuration expects the missing `mold` linker. I’ve restarted it with `lld`, changing only the experiment’s build flags.


## 2026-09-19T00:59:59.000Z — failed

The native Databend Raft-log build now passes with the system linker. Source inspection confirms two boundaries the simulator must preserve: queue backpressure can block the Raft caller synchronously, and the flush thread runs durability callbacks itself. Dropping the WAL also flushes and joins that worker, so object destruction cannot represent a process crash.


## 2026-09-19T01:27:33.000Z — failed

I’ll re-read the latest objective, then resume from the live native probe and cluster build rather than restarting either process.

