# Bug Hunter — Brainstorm (filled)

Inspired by [Antithesis](https://antithesis.com/docs/introduction/how_antithesis_works/).

**GOAL:** Develop a program that explores state spaces efficiently to detect hard-to-reproduce bugs in distributed systems.

**MVP choices:** Rust framework-first, with a minimal ToyQuorum stub to prove APIs.

## 1. State set definition

[Global state](https://www.geeksforgeeks.org/system-design/what-is-the-global-state-of-a-distributed-system/) = all node-local state + in-flight messages + environment (partitions, clocks).

- `SystemState`: semantic snapshot; `fingerprint()` for visited/novelty sets.
- Layers: logical time, per-node (role, crash, pending ops), network queues, environment flags.

## 2. Action set definition

Scheduler decisions: deliver message, faults (partition, crash, drop), workload (client ops), tick.

- `SystemModel::successors` bounds fan-out; `apply` is deterministic given `SimContext`.

## 3. Search algorithm

- Random walk (baseline), BFS with visited set, novelty-guided (feature vectors).
- Stop: step budget, first bug, or coverage plateau.

## 4. Bug detection

- `Oracle` / `CompositeOracle`: invariants on each state.
- `BugReport` ties violation + seed + step + trace.

## 5. Replay artifacts

- JSON trace: `SimConfig`, ordered steps (action + fingerprint), optional violation.
- CLI `replay` re-applies and checks fingerprints.

## 6. Benchmark

- Seed suites × explorer types; metrics: steps, unique fingerprints, time-to-first-bug, replay match.

---

## Real SUT adapter checklist (post-MVP)

1. Implement `SystemState` (`fingerprint`, `summarize`, `features`).
2. Implement `SystemModel` (`initial_state`, `successors`, `apply`).
3. Register `Oracle` invariants (safety first; liveness as best-effort).
4. Map messages into state-held queues; keep nondeterminism only in explorer RNG seed.
5. Run `bug-hunter run` / `bench` / verify saved traces replay.

See crate layout in workspace `Cargo.toml`.