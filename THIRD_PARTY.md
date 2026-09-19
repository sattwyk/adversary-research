# Third-party material

This repository contains reference snapshots and uses dependencies owned by their respective upstream projects. Their original licensing and notices remain in the source archives; this repository does not replace them with a project-wide license.

| Material | Upstream / purpose |
|---|---|
| Databend Meta source snapshot | https://github.com/databendlabs/databend-meta — topology inspection and native storage workload |
| raft-log source snapshots/crate | https://github.com/drmingdrmer/raft-log — actual log encoding, worker integration and recovery |
| chunked-wal source snapshots/crate | https://github.com/drmingdrmer/chunked-wal — batching, write/sync and completion worker |
| OpenRaft crate archive | https://github.com/databendlabs/openraft — source inspection of the pinned dependency |
| SQLite WAL source, two versions | https://sqlite.org/ — affected/fixed correctness-defect comparison; original source notices retained |
| redb dependency | https://github.com/cberner/redb — unchanged production engine, fetched by Cargo |
| Wasmtime dependency | https://github.com/bytecodealliance/wasmtime — host embedding and component/core-Wasm execution |
| corosensei dependency | https://github.com/amanieu/corosensei — native stackful continuation comparison |

Cargo.lock records transitive dependency provenance and checksums. The source archives contain their own license files and manifests. See `archive/reference-snapshots/topology-pins.tsv` for exact Git snapshots and [SOURCES](docs/SOURCES.md) for the broader research bibliography.

The original research code, experiment adapters and written analysis have no project-wide license selected in this private repository.
