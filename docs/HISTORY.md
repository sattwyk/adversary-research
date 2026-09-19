# Research history

This repository records an investigation, including changes of mind and incomplete work. It is not a retrospective claim that the initial architecture was right.

## 1. Original RFC assessment — 18 September 2026

The [RFC](briefs/adversary-rfc.md) proposed a Rust-first platform contract for network, storage, time, entropy, lifecycle and cooperative scheduling, initially using Wasmtime Components/WIT. The [evaluation request](briefs/initial-evaluation-request.md) explicitly asked for adversarial research and falsification.

The [initial response](history/initial-rfc-evaluation.md) compared application-specific simulation, runtime facades, the proposed capability/Wasm boundary, syscall interception and deterministic VMs. It identified production correspondence, synchronous continuations, atomic scheduling units and storage semantics as major uncertainties. Its Wasm assessment predates the executed continuation probes.

## 2. Continuation investigation — 18 September 2026

The [follow-up objective](briefs/1023be70-7526-46cd-ace3-8c81009d0c57.md) asked whether synchronous production call stacks could suspend at effects without an async/reactor rewrite. [Rejection criteria](history/continuation-kill-criteria.md) were recorded before that research.

Executed work established:

- Synchronous Wasmtime component imports can suspend while another Store runs.
- Real, unmodified redb 4.1.0 transaction/recovery code can run through a custom suspended storage backend.
- Native coroutines can preserve synchronous stacks too, but ordinary cancellation executes destructors.

The conclusion changed to conditional **H2**: Wasm suspension expands what can be preserved, but does not preserve arbitrary production runtimes or internal concurrency. See the [full review](research/continuation-review.md), [conversation summary](history/continuation-summary.md), and [completion audit](history/continuation-completion-audit.md). The audit covers that investigation, not the later distributed integration objective.

## 3. Production-topology investigation — started 18 September 2026

The next objective targeted real Databend Meta/OpenRaft/raft-log/chunked-wal behavior. The [protocol](../experiments/topology/protocol.md) records its broader required outcomes and kill criteria. The [topology map](research/production-topology.md) identifies native flush workers, batching, queues, shared payload caches, callbacks, state-machine tasks, snapshot workers and cancellation behavior.

The original local baseline compiled after overriding an unavailable linker. Its recorded package tests passed: 11 unit tests and one recovery integration test. A native observation example was added, but its recorded build failed because `TypeConfigExt` was imported from the wrong module. The original trial log is empty. A native cluster build log also exists; no successful completed cluster experiment is established by that log alone.

Execution stopped before the broad topology objective was completed. In particular, the archive does not establish full deterministic worker preservation, a comparative runtime integration, matched exploration/diagnostic benchmarks, a historical-bug corpus, or qualified power-loss semantics. Revised objective files are preserved in [briefs](briefs/README.md).

## 4. Repository consolidation — 19 September 2026

At the user's request, the research, experiments, original artifacts and unfinished topology material were gathered into a private GitHub repository.

The first commit preserves the available original material. The cleanup adds comments, readable helper names, a separate WAT fixture, a named crash selector, portable scripts, CI and reviewer documentation. It fixes the topology example's import and derives its abandoned-waiter count from execution. Current revalidation is recorded in [STATUS](STATUS.md), independently from the old logs.

Reports are historical documents. Their original code-line counts and snapshot-level observations are not silently rewritten to match the cleaned code. No new bug-discovery, performance or production-correspondence claim is introduced by formatting or successful compilation.
