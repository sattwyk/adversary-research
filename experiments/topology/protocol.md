# Databend execution-topology experiment

Started: 2026-09-18 17:33:42 UTC. Current-source cutoff: 2026-09-18.

Target: Databend Meta actual OpenRaft -> Databend Raft storage -> raft-log -> chunked-wal worker -> write/sync -> callback, plus production state-machine/snapshot/retry paths relevant to P1–P5. Target replacement requires substantive evidence, not convenience.

Before edits: reconstruct tasks/threads/queues/locks/callbacks, shared state, progress and ordering. No replacement of production batching/durability machinery with an atomic simulator operation.

Required outcomes: executable comparison of controlled runtime, suspended Wasm, native fibers, existing worker + modeled I/O, lower-native option; Commonware/Turmoil/MadSim attempts; multi-node native baseline; historical bug corpus with clearly separate mutants; random/perturbed/bounded/semantic exploration with equal compute accounting; code and semantic coverage; replay; ordinary vs causal trace artifacts; storage correspondence including a qualified Linux configuration and crash harness if available; actual effort and code-change accounting.

## Predeclared kill criteria (user-specified)

K1: Replacing worker batching/concurrency architecture fails preservation.
K2: Substantial changes to consensus/storage correctness logic falsify the integration sweet spot.
K3: Equivalent competitor coverage/replay below roughly half the integration effort, without demonstrated extra value, rejects a separate runtime platform.
K4: Easier native preservation with no concrete Wasm benefit makes Wasm optional.
K5: Removing bug-causing workers/dependencies/synchronization/runtime behavior makes a Wasm result invalid.
K6: Historical defects hidden inside atomic units require justified synchronization scheduling or exclusion, not arbitrary instruction simulation.
K7: Economically unrepresentable native storage outcomes reject a correspondence claim.
K8: No measurable semantic-search improvement removes exploration-superiority claims.
K9: No measurable diagnostic improvement removes debugging-superiority claims.
K10: Months of bespoke integration/maintenance comparable to a custom simulator reject a reusable-product claim.

## Safety properties

P1: Client operations acknowledged as committed survive recovery under the declared legal surviving-quorum failure model.
P2: Vote responses are consistent with recovered durable term/vote state; detect contradictory grants through independent request/response/recovery history.
P3: Committed entries remain recoverable in every admitted legal quorum recovery state.
P4: Snapshot publication and rotation/purge crashes cannot remove acknowledged operations from recovery.
P5: Lost response followed by retry cannot cause an application effect disallowed by its actual request-identity/deduplication contract.

Property details and legal fault models will be refined from production contracts before running tests; refinements may not weaken the application's guarantees to obtain a pass.

## Evidence rules

- Preserve pinned clean baselines and all experiment patches.
- Classify dependencies as unchanged, mechanically adapted, semantically reimplemented, or excluded.
- Historical bugs require a verified fixing commit/issue and affected code. Mutants are never historical bugs.
- Record inability to build, inability to preserve, unsupported fault, not tested, and observed failure distinctly.
- SIGKILL is process termination, never proof of power-loss testing.
- Time accounting is measured agent wall-clock activity and command time, not invented equivalent human engineer-hours. Parallel command time is not summed into person-hours.
- Human blinded diagnosis requires actual human participants; any automated proxy must be labeled and cannot satisfy that evidence on its own.
- Full user scope remains active if requirements are unfinished. A worker-only success does not complete this task.

## Environment discovered

Windows host, 16 GiB RAM, 12 logical CPUs; WSL2 archlinux running on a D: backed ext4 virtual disk, approximately 10 GiB memory available to WSL; Docker Linux daemon available. C: has only approximately 8.6 GiB free. Use task-specific Linux scratch/build directories for large build artifacts and copy deliverables to the prescribed outputs directory. Do not modify shared services or existing projects.
