---
name: agent-systems-and-harnesses
version: 1.0.0
format: skill/1.0
description: CIEL's framework for long-lived agents, adversarial harnesses, and headless orchestration.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(orchestrate|harness|manage).*(agent|fleet|gan|gsd|headless)"
    confidence: 0.9
  - pattern: "generator-evaluator loop"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Agent Systems (The Autonomy Layer)

This skill formalizes the operation of complex AI systems, from cloud-hosted workers to adversarial feedback loops.

## Enterprise Agent Ops
1. **Lifecycle**: HARD control over Start, Pause, Stop, and Restart. Hard timeouts on all turns.
2. **Observability**: Track Success Rate, Cost per Task, and Failure Class Distribution.
3. **Least Privilege**: Inject credentials via environment; PROHIBIT broad filesystem or network access without specific audit logs.

## Adversarial Harnesses (GAN-Style)
- **Generator**: Implements features based on spec and evaluator feedback.
- **Evaluator**: Ruthlessly strict QA agent. Uses Playwright to test the LIVE app, not just code.
- **Pass Threshold**: Target weighted score > 7.0 across Design, Craft, and Functionality.
- **Mandate**: Evaluator MUST be a separate instance from Generator to prevent self-praising bias.

## Headless Orchestration (GSD)
- **Spec-Driven**: Launch builds via `gsd headless new-milestone --context spec.md`.
- **Polling**: Use `query` for instant status snapshots (no LLM cost).
- **Blocked State**: If exit code is `10`, query for blocker details and steer or escalate to human.

## Anti-Patterns
- **Self-Evaluation**: Letting an agent approve its own PR or design.
- **Amnesiac Deployment**: Restarting an agent fleet without reading the previous session's persistent memory.
- **Blind Auto-Build**: Running an un-monitored GSD loop for > 30 minutes without a cost-limit cap.
