# ULTRAPLAN — Claude Code

Ultraplan is Claude Code's cloud-based planning + remote execution mode: spin up a fresh environment, run a long-horizon task, return artifacts. Ciel routes here for:

- Large refactors that exceed local context budgets.
- Benchmarks / matrix tests.
- Acquisition workflows that need isolated sandboxes (Tier 3 skills from untrusted origins).
- Multi-day self-improvement sweeps.

## Trigger

Explicit via `/ciel-ultraplan <description>` or automatic when the router's reasoning path reports an estimated token budget > `router.config.context_budget.total_max_tokens * 2`.

## Session Shape

```yaml
ultraplan:
  name: "refactor-imports-monorepo"
  repo: "<git url or local tarball>"
  plan: |
    1. clone
    2. run analysis
    3. propose diffs
    4. open PR / write patch
  budget:
    time: 4h
    tokens: 500k
  on_complete:
    - fetch artifacts
    - Council-gate integration
```

## Artifact Flow

- Ultraplan session writes artifacts (diffs, reports, new `.skill` files) to a durable URL.
- Ciel pulls artifacts, verifies checksums, and routes any new skill through `acquisition/HARMONIZATION.md` + Council.

## Safety

- Ultraplan runs cannot touch the user's `~/.ciel/` directly; outputs are merged locally after Council approval.
- No credentials are transferred to the cloud environment beyond scoped tokens.
- Failure or cost overrun escalates immediately with a report.
