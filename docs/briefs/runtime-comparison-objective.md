You are continuing an empirical systems-research program evaluating whether Adversary should exist as a separate deterministic-testing runtime.

Do not assume that a new runtime is necessary.

The previous experiment produced an important result:

- A real chunked-WAL worker was exercised under native execution.
- Six existing functions gained instrumentation.
- A 226-line controller module was added.
- Databend changed one development-dependency line.
- Consensus, codec, WAL format, and recovery algorithms were not replaced.
- Four documented historical correctness scenarios were reproduced across three fixes.
- Short writes, abandoned waiters, and vote-sync failure were exercised.
- Ten fixed schedules replayed consistently.
- Suspended Wasm was not required.
- Shared-memory concurrent-reader behavior remained outside the controllable schedule.
- No matched Commonware, Turmoil, or MadSim experiment was completed.
- No semantic-exploration advantage, diagnostic advantage, or storage-correspondence claim was established.

Therefore the current strongest hypothesis is only:

> Native semantic checkpoints can preserve a real correctness-sensitive worker and deterministically control meaningful storage/callback orderings.

The next question is whether this justifies a **new runtime**, or whether the useful ideas should live on top of existing execution systems.

# Central question

Determine empirically which of these architectures is sufficient:
```text
A. native execution + semantic checkpoints

B. existing deterministic runtime
   + semantic checkpoints/effect model

C. new Adversary deterministic runtime

D. optional Wasm execution backend
```

Do not favor C.

The desired result may be that C is unnecessary.

---

# 1. Freeze the existing experimental target

Use the same pinned chunked-WAL / Databend integration from the previous experiment.

Do not change the target merely to make a competitor easier to integrate.

Reuse the same:

- production worker;
- historical regression scenarios;
- synthetic fault cases;
- checker/oracle;
- traces;
- replay format where possible.

The comparison must be matched.

---

# 2. Establish the native-checkpoint baseline

Treat the existing experiment as architecture A.

Document precisely:

- modified production functions;
- instrumentation LOC;
- controller LOC;
- execution contexts controlled;
- execution contexts uncontrolled;
- fault points;
- replay mechanism;
- reproduced historical scenarios;
- excluded historical scenarios.

Measure:

- runs per second;
- replay reliability;
- memory consumption;
- integration LOC;
- wall-clock implementation effort where reconstructable.

This is the baseline competitors must beat or improve upon.

---

# 3. Implement the smallest viable Turmoil integration

Do not redesign the application around Turmoil.

Use Turmoil only where its existing abstractions naturally apply.

Determine:

1. Can the existing WAL worker remain?
2. Can its task/worker scheduling become deterministic?
3. Can the same semantic checkpoints remain?
4. Can the current storage-fault model be connected without rewriting WAL logic?
5. Can all previously reproduced regressions still be triggered?
6. Does Turmoil expose additional relevant schedules?
7. What remains uncontrolled?

Do not count a test that replaces the WAL worker with simulated durable storage as equivalent.

Record all production modifications and integration code.

---

# 4. Implement the smallest viable Commonware comparison

Commonware may not be a drop-in runtime for the target.

Do not force the entire Databend application into Commonware.

Instead determine the minimum architecture required to provide equivalent control.

Ask separately:
```text
Can Commonware execute this production worker mostly unchanged?
```

and:
```text
Could Adversary's semantic storage/exploration layer reasonably be implemented on top of Commonware?
```

These are different questions.

If the direct port requires large rewrites but Commonware could host the reusable semantic engine cleanly, record that outcome.

Measure the same quantities as the native baseline.

---

# 5. Evaluate MadSim similarly

Attempt the smallest honest MadSim integration.

Pay special attention to:

- Tokio substitution;
- worker/thread behavior;
- filesystem model limitations;
- transitive dependencies;
- task scheduling;
- replay.

If the production worker uses execution behavior MadSim cannot retain without rewriting it, document this as a concrete compatibility boundary.

Do not compensate by simplifying the target.

---

# 6. Separate runtime value from model value

The previous experiment combines several ideas that must now be separated:
```text
semantic checkpoints
deterministic scheduling
storage fault model
causal tracing
exploration strategy
execution isolation
Wasm continuation preservation
```

For each reproduced failure identify which capability was actually necessary.

Construct a dependency table.

Example:

| Failure          | deterministic scheduler | semantic checkpoint | storage model | Wasm | custom runtime |
| ---------------- | ----------------------- | ------------------- | ------------- | ---- | -------------- |
| ack-before-sync  | yes                     | yes                 | yes           | no   | ?              |
| abandoned waiter | yes                     | yes                 | partial       | no   | ?              |

The purpose is to discover whether "Adversary" is really one architecture or several independently useful libraries.

---

# 7. Attack the native-checkpoint design

The native integration looked surprisingly cheap.

Try to break it.

Investigate whether instrumentation alters:

- queue timing;
- locking;
- synchronization;
- compiler optimization;
- worker lifecycle;
- callback timing;
- ownership;
- cancellation;
- memory ordering.

Determine whether checkpoints are observational or whether they materially perturb the execution topology.

Measure the overhead with checkpoints:
```text
disabled
enabled but passive
actively scheduled
```

If native checkpoints preserve enough behavior, they may dominate Wasm/runtime replacement for existing Rust infrastructure.

---

# 8. Extend scheduling to the known concurrent-reader exclusion

The previous experiment found at least one documented concurrency scenario that worker-level checkpoints cannot represent.

Use that scenario as a boundary experiment.

Do not immediately implement instruction-level scheduling.

First identify the minimum additional scheduling events needed.

Candidates:
```text
lock acquire/release
channel send/receive
condition wake/wait
atomic publication
specific shared-state read/write
explicit project checkpoint
```

Test progressively:

### Level 0

Current worker checkpoints.

### Level 1

Synchronization-operation checkpoints.

### Level 2

A small number of targeted shared-state checkpoints.

### Level 3

Shuttle/Loom-style controlled synchronization where feasible.

Determine the **coarsest scheduling model** that reproduces the documented race.

Measure how much state-space growth each additional level creates.

The goal is not universal thread simulation.

The goal is to discover whether a small semantic scheduling vocabulary captures substantially more real bugs.

---

# 9. Compare against Shuttle/Loom for this race

If the concurrent-reader scenario can be reduced to a shared-memory component:

- attempt a Shuttle reproduction;
- attempt Loom if the primitives and memory model make that meaningful.

Determine whether the appropriate architecture is compositional:
```text
distributed/effect exploration
        +
Shuttle/Loom concurrency tests
```

rather than one simulator attempting to control every interleaving.

A result where different tools cover different layers is acceptable.

---

# 10. Perform the first exploration experiment

The previous experiment only replayed fixed schedules.

That does not establish deterministic simulation testing value.

Using the existing controllable worker, implement at least three search strategies:

## Uniform/random choice

Choose among currently eligible semantic events.

## PCT/bounded-delay style scheduling

Where applicable.

## Semantic targeting

Prioritize states such as:
```text
callback eligible before sync completion

multiple WAL requests pending

write effected but waiter unresolved

sync finished but completion withheld

shutdown while callbacks pending

retry while previous effect exists
```

Use intentionally seeded mutants whose failure conditions require different schedule structures.

Do not tune one strategy separately for each mutant.

Run equal execution budgets.

Measure:
```text
probability of finding failure
executions to first failure
CPU time to first failure
unique failing traces
semantic states reached
```

This is the first actual test of Adversary's exploration thesis.

---

# 11. Test trace value without a large user study

A full blinded developer experiment is premature.

Instead establish whether semantic traces objectively remove debugging work.

For each reproduced regression generate:

### Ordinary trace

Existing application logs + stack trace.

### Semantic trace

Operation/effect/completion/persistence/scheduling causal chain.

Then independently reconstruct the minimum causal explanation from each.

Measure:
```text
number of events inspected
number of unrelated events
length of minimal explanation
ability to identify violated ordering directly
```

This is weaker than a human study but sufficient to decide whether deeper investment is justified.

---

# 12. Decide whether Wasm should remain in the project

Do not run another Wasm experiment unless one of the matched runtime experiments produces a concrete need.

Wasm should remain relevant only if a target contains valuable synchronous call stacks that:
```text
cannot be controlled naturally by the native/runtime approach
AND
can be compiled to Wasm without removing the relevant behavior.
```

Otherwise classify Wasm as an optional backend.

The earlier redb result already proves feasibility.

More mechanism demonstrations do not constitute new evidence.

---

# 13. Architecture decomposition

At the end determine whether the project is best represented as:

## Architecture A
```text
Adversary runtime
+ scheduler
+ environment
+ Wasm
```

## Architecture B
```text
Adversary semantic testing library
    ├─ effects
    ├─ storage models
    ├─ traces
    ├─ exploration
    └─ checkpoints

execution backends:
    native
    Turmoil
    Commonware
    MadSim
    optional Wasm
```

## Architecture C
```text
extensions/contributions
to an existing deterministic runtime
```

## Architecture D
```text
application-specific testing toolkit
rather than reusable platform
```

Prefer whichever the evidence supports.

---

# 14. Predeclared decision rules

A separate Adversary runtime is **not justified** if:

1. native checkpoints preserve equivalent production behavior;
2. an existing runtime can host the semantic model with comparable effort;
3. historical bugs do not require unique Adversary execution machinery;
4. Wasm is unnecessary for the target population;
5. semantic exploration works independently of the execution backend.

A separate runtime becomes more defensible if:

1. existing runtimes structurally cannot preserve important execution topology;
2. native checkpoints cannot provide deterministic replay safely;
3. the same semantic scheduler works across native + Wasm execution;
4. real failures require operation semantics that existing runtimes cannot expose without major redesign.

---

# Required final structure

End with:

1. **Native checkpoint baseline**
2. **Turmoil integration result**
3. **Commonware integration result**
4. **MadSim integration result**
5. **Which production behavior each preserved**
6. **Which historical regressions each reproduced**
7. **Concurrent-reader boundary experiment**
8. **Minimum scheduling granularity required**
9. **Exploration experiment results**
10. **Semantic trace result**
11. **What capabilities actually require Adversary**
12. **What capabilities are backend-independent**
13. **Does Wasm still earn a central role?**
14. **Does a new runtime still earn a central role?**
15. **Should this become a library, runtime extension, or standalone platform?**
16. **The strongest remaining hypothesis**

Do not give a motivational conclusion.

Do not count incomplete competitor experiments as evidence for Adversary.

Do not treat compilation or deterministic replay alone as sufficient product differentiation.

The objective is to determine whether the useful result from the WAL experiment implies a new systems architecture—or merely a useful set of testing primitives.