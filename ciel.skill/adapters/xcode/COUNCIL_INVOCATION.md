# COUNCIL_INVOCATION — Xcode Coding Assistant

Xcode Coding Assistant does not expose native nested subagents through the adapter. Ciel runs Council of Five deliberation inline and sequentially.

## Topology

```yaml
runtime: xcode
council_topology: sequential-inline
parallel: false
blocking_gate: inline_preflight
```

## Flow

1. State the proposed action and affected files.
2. Classify risk using `risk/CLASSIFICATION.md`.
3. Evaluate in order: Coherence, Capability, Safety, Efficiency, Evolution.
4. Stop immediately on Safety veto for critical risk.
5. Synthesize the decision and ask the user when required by the escalation ladder.
6. Execute through Xcode MCP or shell only after approval conditions are satisfied.

## Required For

- Changes to locked Ciel core files.
- Installing or removing capabilities.
- Destructive filesystem operations.
- Broad rewrites that affect project behavior.
- Sandbox escape or elevated shell execution.

