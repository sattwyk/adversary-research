# Preregistered decision criteria — 18 September 2026

Scope: investigate the full goal-objective.md, not implement a general simulator.

Before new research, reject or radically narrow the corresponding architecture if:
1. Retaining synchronous correctness code requires extensive async/CPS/manual reactor rewrites.
2. Wasmtime cannot suspend imports and let the host choose deterministic effects and resume order.
3. The Wasm build removes libraries, concurrency, or memory behavior responsible for the target bugs.
4. Native fibers or existing runtimes provide equivalent useful coverage with materially lower total cost.
5. Real safety regressions require interleavings hidden inside the proposed scheduling unit.
6. Native storage traces cannot be qualified against maintainable abstract profiles.
7. Commonware/Turmoil/MadSim achieve equivalent coverage at lower integration and maintenance cost.
8. Supporting required execution semantics amounts to enough runtime/OS emulation that native syscall/VM execution is simpler overall.

Treat counterexamples as scope changes or rejection, not automatically as future work. Distinguish source/documentation evidence from executed experiments. Do not claim a port or benchmark was performed unless it was.

Required evidence: current Wasmtime APIs plus implementation/tests; actual call graphs in at least three targets; all named ecosystem inspection; scheduling and cancellation semantics; precise 12-step suspended commit design; four prototype protocols; exact 12-section final synthesis.
