# Completion audit — 18 September 2026

Goal objective read in full. Original RFC had been read in full in the preceding investigation. No subagents used.

| Requirement | Evidence/deliverable |
|---|---|
| Compare A–E | Report sections 3, 5, 8; per-target consequences for redb, Databend/OpenRaft, SQLite |
| Inspect named production implementations | Section 5 source-call-path table: Pebble, RocksDB, redb, SQLite, etcd, Databend/OpenRaft, FoundationDB, TigerBeetle, PostgreSQL |
| Favorable, sync seam, hostile cases | Section 5; source inspection explicitly distinguished from executed port |
| Actual 12-step suspended-call mechanism | Executed component probe, 404 selected schedule runs; section 4 |
| Real production code preservation | Unmodified redb 4.1.0 core Wasm, 54-line wrapper, 114-line host; nine crash scenarios repeated ten times |
| Native alternative | Executed 43-line corosensei probe; 100 repetitions; destructor distinction |
| Code integrity | Registry archive SHA matches Cargo.lock; 71 packaged files match compiled crate sources |
| Wasmtime current mechanisms/internals | Version 48.0.2 linker, fibers, Store async, concurrent component runtime, cancellation tests, Config; P3 docs |
| Ownership, reentry, reset, replay, snapshots, threads | Section 4; no unsupported snapshot/fiber-cloning claim |
| Production correspondence and language/runtime differences | Sections 2, 5, 6 |
| Scheduling granularity | Section 3 table, split-lock example, fuel limitations |
| Crashes during execution | Section 2 equivalence conditions and counterexamples |
| Cancellation distinctions and adapters | Section 9, Tokio/io_uring/Go/POSIX/Wasmtime distinctions |
| H1–H4 conclusion | H2 conditional verdict, H1/D and H4 fallback criteria; sections 1, 11, 12 |
| Direct competitor comparison | Section 7 six concrete tests, current Commonware/Turmoil/MadSim references |
| Prior art beyond immediate competitors | Sthread/SimGrid, CHESS, Asyncify, corosensei, Boost, FrostDB, PCT |
| Exploration and causal explanation | Section 9; no claim uniquely enabled by Wasm |
| Strongest revised architecture | Section 9, bounded SDK/backends and native conformance |
| Four minimum experiments, quantitative gates | Section 10; stated proposed limits, not fabricated measurements |
| Known excluded real safety bug | SQLite 3.51.2/3.51.3 wal.c diff inspected; section 8/D; not claimed reproduced locally |
| Before-research kill criteria | work/continuation-kill-criteria.md, copied into probe bundle; current status section 11 |
| Required final 12 sections | Full report has exactly 12 requested numbered sections; final response will use same titles |
| Reproducibility | Source, Cargo.lock, recorded stdout, integrity, README in outputs/continuation-probes.zip |

Final probes passed after recovery-oracle correction (unexpected errors fail rather than being treated as missing data). No throughput measurements, full Databend/SQLite port, native filesystem qualification, broad real-bug rediscovery, or complete four-way benchmark are claimed. Those are explicitly specified future deciding experiments; the current request is investigation plus minimum-prototype design, not implementation of every proposed experiment.

Report approximately 8,000 words. Artifacts contain no dependency caches or native executables. Source package is approximately 23 KB compressed.
