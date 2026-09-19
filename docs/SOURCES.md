# Research source index

Primary references collected in the September 18, 2026 research reports. This is a navigation index, not a new source-validation run on September 19. Pinned local snapshots are listed in `archive/reference-snapshots/topology-pins.tsv`; mutable upstream URLs retain the limitations described in the reports.

## antithesis.com

- [2026 Raft results](https://antithesis.com/blog/2026/finding-bugs-in-raft-implementations/)

- [Antithesis reproduction](https://antithesis.com/blog/2026/wal-reset-bug/)

- [Hypervisor architecture](https://antithesis.com/blog/deterministic_hypervisor/)

- [8192, 8448) write completed response “committed” emitted power loss selected a permitted state without that range recovery discarded transaction 417 client history violates durability ```  That is substantially better than an unexplained failed assertion and a large syscall trace.  However, the explanation must distinguish:  - No durability barrier was established. - The model selected an outcome where the data was absent. - Native execution has reproduced that outcome. - A stronger synchronization placement prevents the failure.  These are different claims. Data can persist without a sync; lack of a sync alone does not prove that a particular native execution lost it.  Build explanations from an explicit dependency graph, then validate them through counterfactual replays. Do not infer causality from event proximity.  This could justify meaningful integration work for storage teams. But debugging quality needs a user experiment: give engineers equivalent failures from Adversary and a baseline, then measure time to correct diagnosis and repair.  Antithesis already offers replay and the ability to revisit executions with richer diagnostics. Adversary needs better *semantic explanations*, not merely another replay UI. [Antithesis debugging guidance](https://antithesis.com/docs/best_practices/optimizing/)

- [environment documentation](https://antithesis.com/docs/configuration/the_antithesis_environment/)

- [Antithesis exploration](https://antithesis.com/docs/introduction/how_antithesis_works/)

- [test integration](https://antithesis.com/docs/product/writing_tests/)

## apple.github.io

- [testing documentation](https://apple.github.io/foundationdb/testing.html)

## arxiv.org

- [Sthread paper](https://arxiv.org/abs/2002.06223)

- [Mallory](https://arxiv.org/abs/2305.02601)

- [Themis preprint](https://arxiv.org/html/2608.01135v1)

## component-model.bytecodealliance.org

- [Language support](https://component-model.bytecodealliance.org/language-support.html)

- [Go components](https://component-model.bytecodealliance.org/language-support/building-a-simple-component/go.html)

## docs.kernel.org

- [XFS delayed logging design](https://docs.kernel.org/filesystems/xfs/xfs-delayed-logging-design.html)

## docs.rs

- [Commonware deterministic runtime 2026.9.0](https://docs.rs/commonware-runtime/2026.9.0/commonware_runtime/deterministic/index.html)

- [Commonware Blob](https://docs.rs/commonware-runtime/2026.9.0/commonware_runtime/trait.Blob.html)

- [Runtime documentation](https://docs.rs/commonware-runtime/latest/commonware_runtime/deterministic/index.html)

- [Blob](https://docs.rs/commonware-runtime/latest/commonware_runtime/trait.Blob.html)

- [Commonware Storage](https://docs.rs/commonware-runtime/latest/commonware_runtime/trait.Storage.html)

- [redb backend interface](https://docs.rs/redb/latest/redb/trait.StorageBackend.html)

- [Tokio `select!` documentation](https://docs.rs/tokio/latest/tokio/macro.select.html)

- [Tokio `spawn_blocking`](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)

- [Tokio JoinHandle](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html)

- [Wasmtime Config, including RRConfig and execution controls](https://docs.rs/wasmtime/48.0.2/wasmtime/struct.Config.html)

- [Wasmtime `func_wrap_async`](https://docs.rs/wasmtime/latest/wasmtime/component/struct.LinkerInstance.html)

## docs.wasmtime.dev

- [Wasmtime asynchronous execution](https://docs.wasmtime.dev/examples-async.html)

- [Wasmtime determinism guidance](https://docs.wasmtime.dev/examples-deterministic-wasm-execution.html)

## dropbox.tech

- [Dropbox technical account](https://dropbox.tech/infrastructure/-testing-our-new-sync-engine)

## emscripten.org

- [Asyncify](https://emscripten.org/docs/porting/asyncify.html)

## github.com

- [corosensei](https://github.com/amanieu/corosensei)

- [Source and documentation](https://github.com/awslabs/shuttle)

- [componentize-py](https://github.com/bytecodealliance/componentize-py)

- [Wasmtime 48.0.2](https://github.com/bytecodealliance/wasmtime/releases/tag/v48.0.2)

- [Workspace](https://github.com/databendlabs/databend-meta)

- [Raft log integration](https://github.com/databendlabs/databend-meta/blob/main/crates/server/raft-log/src/impl_raft_log_storage.rs)

- [Repository](https://github.com/facebookexperimental/hermit)

- [Filesystem source](https://github.com/madsim-rs/madsim/blob/main/madsim/src/sim/fs.rs)

- [Loom](https://github.com/tokio-rs/loom)

- [repository](https://github.com/tokio-rs/turmoil)

## go.dev

- [Go’s explanation](https://go.dev/blog/testing-time)

## gvisor.dev

- [gVisor architecture](https://gvisor.dev/docs/architecture_guide/intro/)

## man7.org

- [close(2)](https://man7.org/linux/man-pages/man2/close.2.html)

- [write(2)](https://man7.org/linux/man-pages/man2/write.2.html)

- [io_uring cancellation](https://man7.org/linux/man-pages/man3/io_uring_prep_cancel.3.html)

## people.ucsc.edu

- [Molly](https://people.ucsc.edu/~palvaro/molly.pdf)

## pkg.go.dev

- [Go context](https://pkg.go.dev/context)

- [FrostDB DST instructions](https://pkg.go.dev/github.com/polarsignals/frostdb/dst)

## raw.githubusercontent.com

- [FoundationDB disk queue](https://raw.githubusercontent.com/apple/foundationdb/7.4.7/fdbserver/DiskQueue.actor.cpp)

- [Wasmtime concurrent component runtime](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/crates/wasmtime/src/runtime/component/concurrent.rs)

- [Component linker implementation](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/crates/wasmtime/src/runtime/component/linker.rs)

- [Fiber disposal](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/crates/wasmtime/src/runtime/fiber.rs)

- [Store async implementation](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/crates/wasmtime/src/runtime/store/async_.rs)

- [async cancellation tests](https://raw.githubusercontent.com/bytecodealliance/wasmtime/v48.0.2/tests/all/async_functions.rs)

- [StorageBackend](https://raw.githubusercontent.com/cberner/redb/v4.1.0/src/db.rs)

- [redb transactions](https://raw.githubusercontent.com/cberner/redb/v4.1.0/src/transactions.rs)

- [cached-file implementation](https://raw.githubusercontent.com/cberner/redb/v4.1.0/src/tree_store/page_store/cached_file.rs)

- [page manager](https://raw.githubusercontent.com/cberner/redb/v4.1.0/src/tree_store/page_store/page_manager.rs)

- [Pebble commit pipeline](https://raw.githubusercontent.com/cockroachdb/pebble/master/commit.go)

- [Pebble log writer](https://raw.githubusercontent.com/cockroachdb/pebble/master/record/log_writer.go)

- [crashable memory filesystem](https://raw.githubusercontent.com/cockroachdb/pebble/master/vfs/mem_fs.go)

- [Pebble VFS source](https://raw.githubusercontent.com/cockroachdb/pebble/master/vfs/vfs.go)

- [Databend Raft storage](https://raw.githubusercontent.com/databendlabs/databend-meta/main/crates/server/raft-log/src/impl_raft_log_storage.rs)

- [chunked-wal 0.2.0 worker](https://raw.githubusercontent.com/drmingdrmer/chunked-wal/v0.2.0/src/wal/flush_worker.rs)

- [etcd WAL](https://raw.githubusercontent.com/etcd-io/etcd/main/server/storage/wal/wal.go)

- [RocksDB write implementation](https://raw.githubusercontent.com/facebook/rocksdb/main/db/db_impl/db_impl_write.cc)

- [Go 1.26 WASI syscall implementation](https://raw.githubusercontent.com/golang/go/go1.26.0/src/syscall/fs_wasip1.go)

- [MadSim filesystem implementation](https://raw.githubusercontent.com/madsim-rs/madsim/main/madsim/src/sim/fs.rs)

- [runtime implementation](https://raw.githubusercontent.com/polarsignals/frostdb/main/dst/runtime/run.go)

- [PostgreSQL WAL](https://raw.githubusercontent.com/postgres/postgres/REL_18_STABLE/src/backend/access/transam/xlog.c)

- [SQLite WAL](https://raw.githubusercontent.com/sqlite/sqlite/version-3.51.2/src/wal.c)

- [3.51.3 implementation](https://raw.githubusercontent.com/sqlite/sqlite/version-3.51.3/src/wal.c)

- [TigerBeetle storage](https://raw.githubusercontent.com/tigerbeetle/tigerbeetle/main/src/storage.zig)

- [simulated storage source](https://raw.githubusercontent.com/tigerbeetle/tigerbeetle/main/src/testing/storage.zig)

- [README](https://raw.githubusercontent.com/tokio-rs/loom/master/README.md)

- [Filesystem README](https://raw.githubusercontent.com/tokio-rs/turmoil/main/crates/turmoil-fs/README.md)

- [Turmoil filesystem implementation and IoLatency notes](https://raw.githubusercontent.com/tokio-rs/turmoil/main/crates/turmoil-fs/src/lib.rs)

## research.cs.wisc.edu

- [ALICE research](https://research.cs.wisc.edu/adsl/Publications/alice-osdi14.html)

## rr-project.org

- [rr](https://rr-project.org/)

## simgrid.org

- [SimGrid model checking](https://simgrid.org/doc/latest/Tutorial_Model-checking.html)

## sqlite.org

- [SQLite’s technical description](https://sqlite.org/wal.html#walreset)

## static.usenix.org

- [WiDS](https://static.usenix.org/event/nsdi07/tech/full_papers/liu/liu_html/WiDSChecker.html)

## tailscale.com

- [Tailscale’s incident account](https://tailscale.com/blog/sqlite-wal-reset-bug)

## teavm.org

- [TeaVM overview](https://teavm.org/docs/intro/overview.html)

## tigerbeetle.com

- [Vortex technical account](https://tigerbeetle.com/blog/2025-02-13-a-descent-into-the-vortex/)

- [TigerBeetle’s account](https://tigerbeetle.com/blog/2025-06-06-fuzzer-blind-spots-meet-jepsen/)

- [Protocol-aware DST](https://tigerbeetle.com/blog/2026-08-20-protocol-aware-dst/)

## ucare.cs.uchicago.edu

- [FlyMC](https://ucare.cs.uchicago.edu/pdf/eurosys19-flyMC.pdf)

## unsat.cs.washington.edu

- [Ferrite paper](https://unsat.cs.washington.edu/papers/bornholt-ferrite.pdf)

## wasi.dev

- [WASI P3 releases](https://wasi.dev/releases/wasi-p3)

## www.boost.org

- [Boost.Fiber](https://www.boost.org/library/latest/fiber/)

## www.foundationdb.org

- [Paper](https://www.foundationdb.org/files/fdb-paper.pdf)

## www.kernel.org

- [Ext4 journal documentation](https://www.kernel.org/doc/html/latest/filesystems/ext4/journal.html)

## www.microsoft.com

- [CHESS implementation and scheduling approach](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/chess-osdi2008-chess.pdf)

- [PCT paper](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/paper-83.pdf)

## www.nvmexpress.org

- [NVMe specification](https://www.nvmexpress.org/wp-content/uploads/NVM-Express-1_4c-2021.06.28-Ratified.pdf)

## www.polarsignals.com

- [Technical account](https://www.polarsignals.com/blog/posts/2024/05/28/mostly-dst-in-go)

## www.postgresql.org

- [PostgreSQL change and rationale](https://www.postgresql.org/message-id/E1gObRL-00026a-N7@gemulon.postgresql.org)

## www.rfc-editor.org

- [TCP specification](https://www.rfc-editor.org/rfc/rfc9293.html)

## www.usenix.org

- [Legolas](https://www.usenix.org/conference/nsdi24/presentation/wu-haoze)

- [SAMC](https://www.usenix.org/conference/osdi14/technical-sessions/presentation/leesatapornwongsa)

- [CrashMonkey/Ace paper](https://www.usenix.org/conference/osdi18/presentation/mohan)

- [MODIST](https://www.usenix.org/legacy/events/nsdi09/tech/full_papers/yang/yang_html/)
