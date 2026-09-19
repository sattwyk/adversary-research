# Provenance and archive policy

## Included material

- The original user-supplied RFC and research objectives.
- The two completed user-facing analysis responses, exported as Markdown, plus the longer research report.
- Original continuation source, lockfiles, result logs and integrity record; the original delivered ZIP is preserved byte-for-byte.
- Production-topology documentation, protocol, timing ledger, native experiment source, Cargo.lock, successful baseline logs and failed/incomplete build logs.
- Source snapshots used during research: pinned Databend Meta, raft-log and chunked-wal Git trees; downloaded crate archives; affected/fixed SQLite WAL source files.
- Cleaned, commented experiment code; repeatable scripts; current verification evidence.

## Source reconstruction

`archive/reference-snapshots/topology-pins.tsv` contains upstream URLs and commit IDs. The `.tar.gz` files were generated from those local Git commits with `git archive`, retaining upstream notices. The working-tree patches were checked ignoring line-ending-only changes; all three are empty in this snapshot. The added topology example and generated Cargo.lock are stored separately and overlaid by `scripts/prepare-topology.sh`.

Earlier versions used during the continuation research remain as `.crate` archives. Their embedded manifests and license files preserve their original provenance. Source snapshots are reference material, not a claim that their entire contents were executed in the probes.

The original redb integrity record verifies the registry archive against Cargo.lock and compares all 71 packaged files against compiled sources. New binaries have their own dated hashes. Formatting/wrapper changes can change binary hashes without changing the upstream dependency.

## Historical versus current files

The first Git commit is the consolidation baseline. The `archive/continuation-probes-original.zip` file is the original delivery. `experiments/` contains maintained, readable copies. `results/2026-09-18/` retains original stdout/build evidence, including empty and failed outputs; new runs use a different date.

Text files are normalized to LF in Git. Historical machine-local links in the exported continuation summary were redirected to repository artifacts for navigation. Historical paths inside logs/scripts remain provenance; executable maintained scripts resolve paths relative to this checkout.

No tool caches, target directories, credentials, authentication configuration, raw session files or internal reasoning were copied. Research analysis here means the written user-facing reports, evidence and experiment design. Public source archives are retained; large rebuildable dependencies are fetched through the locked manifests.

The snapshot records unfinished topology work honestly. It does not resume or complete every broader experiment described in the archived objectives.
