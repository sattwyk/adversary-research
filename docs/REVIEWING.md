# Reviewer guide

## Recommended review order

1. Read the root README and [STATUS](STATUS.md) to establish which claims have evidence.
2. Read [the kill criteria](history/continuation-kill-criteria.md), then the [review](research/continuation-review.md).
3. Inspect `experiments/continuations/host/fixtures/commit.wat` and `host/src/main.rs` together. Follow one request from submission through effect, completion, and guest resumption.
4. Inspect the redb guest adapter, host crash driver, and recovery oracle together.
5. Compare the native fiber's Drop behavior with the Wasm kill path.
6. Read the production-topology map before judging the unfinished Databend integration.
7. Compare the first commit with the cleaned code. Use the original ZIP/logs when checking historical measurements.

## Questions worth challenging

- Does an operation outlive its waiter? Does dropping a Future erase anything already effected?
- Are effect, durable state, completion readiness and guest resumption distinct?
- Does a crash run guest cleanup? Is host cleanup being confused with guest cleanup?
- Are guest buffers copied/owned safely across suspension and destruction?
- Does the recovery oracle reject unexpected errors, rather than count corruption as the old state?
- Does the real engine's sync path run, or has it been replaced by an atomic high-level operation?
- Which contexts can run while a Store is blocked? Which production peers/shared memory are absent?
- Does a replay comparison check only trace labels, actual state, or both? The probes contain both trace comparisons and selected state assertions, not a general state-equivalence checker.
- Is the storage profile qualified against a real filesystem? Here, it is not.
- Is a graceful close or SIGKILL being misreported as sudden power loss?
- Are a successful baseline build, a native workload, and a deterministic integration kept distinct?

## Running and changing code

Run `bash scripts/run-probes.sh`. Keep the semantic experiments small and inspectable. Avoid introducing a generic runtime merely to reduce duplicated code across deliberately different probes.

When changing a fault scenario, document its allowed recovered states and update the dated evidence. Do not change an oracle just to obtain a passing run. Historical bugs and intentionally introduced mutants must be labeled separately.

The topology script has its own pinned source preparation. It should not modify an existing upstream checkout or depend on an author's home directory. Its observation logger reads an upstream diagnostic string; that is fragile and not a permanent tracing API.

## Evidence needed before stronger claims

The experiment protocols in review section 10 remain the decision plan: matched continuation comparisons, real storage qualification, preservation of the real distributed worker topology, and an explicit excluded concurrency defect. Passing the current probes is not completion of those experiments.
