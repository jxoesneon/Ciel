---
name: double-loop
description: Double agentic loop — an outer supervisor loop (decompose, packetize, dispatch, independently verify, correct, replan) driving a bounded pool of hot-swappable inner worker loops. Use for worklists too large for one context, especially "implement every remaining issue" mandates.
license: MIT
metadata:
  ciel-version: 1.0.0
  ciel-extension: ciel.yaml
---

# CIEL ADAPTATION: Double Agentic Loop

Two nested loops on different cadences, separated by a single structured channel:

- **Outer loop (Supervisor — this session)**: decomposes the goal into file-disjoint **Work Packets**, maintains the slot pool, **independently verifies** every returned result (never accepts the worker's own say-so), issues **Correction Packets** on failure, and replans when mid-run evidence changes the picture. The outer loop holds all global state: the worklist, the file-ownership map, the integration order.
- **Inner loop (Worker — one ephemeral subagent per packet)**: runs its own tool-use loop on exactly one packet — implement, self-verify, report a **Structured Result**. The inner loop holds only local state. It never sees the whole plan.

The loops are connected by two channels only: the packet going down, and the structured result coming up. The interrupt channel is the worker's `BLOCKED`/`NEEDS_CONTEXT` report plus the supervisor's own file-state reads.

## The Core Protocol

### Pre-dispatch hooks (run once per activation)

1. **Worklist freeze**: enumerate every open item; assign each a stable WP id.
2. **File-ownership map**: for every WP, list the exact files it may touch. Two WPs may not run concurrently if their file sets intersect. Shared files force sequencing or merging into one WP.
3. **Tree-state check**: `git status` clean or accounted-for; note in-flight work from other sessions and route around it.
4. **Wave plan**: order WPs so sweeping refactors (cross-cutting type/DI changes) run LAST, serially, after all localized packets land.

### Per-packet hooks

- **Packet author hook**: every packet carries — scope, exact target files, acceptance criteria, evidence citations (file:line), commit-message convention, and the "NO PUSH / NO unrelated files" rule. Pass only what the worker needs.
- **Completion hook (mandatory independent verification)**: on each worker return —
  1. `git log`/`git diff` — the commit exists and touches ONLY the packet's files.
  2. Run the packet's own verification evidence (the tests it claims) — spot-run them yourself; do not trust the report alone.
  3. Confirm acceptance criteria against the actual diff.
  4. Close the tracked issue with a resolution comment citing the commit.
  5. Then — and only then — hot-swap the freed slot to the next packet.
- **Failure hook**: on `BLOCKED`/test failure/partial work — diagnose from the diff, author a **Correction Packet** (narrow, specific, referencing the failing evidence), re-dispatch the SAME slot. Never let a failed packet silently drop.
- **Integration hook**: after each wave, run the affected test files together and `dart analyze` (or project equivalent) across the union of changed files — cross-packet conflicts surface here, not in the worker loops.

### Slot pool (hot-swap model)

- Fixed N slots (default 5). Each slot holds: `packet_id → worker_id → status`.
- On worker completion notification: run the completion hook, then immediately dispatch the next eligible packet into the freed slot. Slots are interchangeable — eligibility is determined only by file-ownership and sequencing, never by which slot freed.
- A worker that stalls without progress is killed and its packet re-dispatched with tighter scope.

### Escalation paths

- Packet fails twice → escalate to Council of Five for a design decision.
- Verification reveals the packet spec was wrong → supervisor owns the spec fix, re-dispatches; do not patch the worker's output from the outer loop unless trivially small or correctness-critical.
- Cross-session interference (another agent editing owned files) → freeze that WP, replan.

## Anti-patterns

- Implementing in the outer loop (context pollution) — the supervisor's job is packets and verification.
- Sequential dispatch when file sets are disjoint (wasted parallelism).
- Parallel dispatch when file sets intersect (race conditions, half-merged fixes).
- Trusting worker reports without the completion-hook evidence run.
- Releasing a sweeping refactor concurrently with localized packets on the same files.
