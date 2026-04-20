# PLAN_MODE — Gemini CLI

Plan mode is a read-only mode: writes and destructive actions are replaced by descriptions of what *would* happen. Ciel uses Plan mode as her default pre-flight for high-risk operations.

## When Ciel Enters Plan Mode

- Risk classification ≥ mid AND user has `risk.config.plan_mode_gate: true`.
- Operation touches a locked core file.
- Operation is a skill acquisition with Tier 3 source (web-extracted).
- Explicit operator request: `/ciel --plan <request>`.

## Flow

1. Router proposes a plan.
2. Ciel switches host to Plan mode.
3. Plan executes — every tool invocation returns a *describe* rather than an effect.
4. Ciel collects the describes, builds a preview.
5. If preview passes Council-quick judge, she exits Plan mode and re-runs for real.
6. If preview flags concern, escalate or re-plan.

## Fallback on Other Runtimes

Claude Code does not have native Plan mode. For Claude Code, Ciel simulates by registering a deny-all `PreToolUse` hook for the session then capturing every `deny` message as the plan preview. See `adapters/claude_code/HOOKS.md`.

## Output

Plan preview saved to `.ciel/traces/plan-<timestamp>.md` for audit and Council inspection.

## Cost

Plan mode uses tokens without effect. Ciel budgets Plan-mode runs at `router.config.plan_mode.budget_tokens` per invocation.
