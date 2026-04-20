# LLM_JUDGE

Lightweight self-audit for mid-risk operations. Ciel audits Ciel.

## When Called

- `risk/CLASSIFICATION.md` returns `mid`.
- `router/ROUTER.md` composition has any `mid` sub-step.
- Sensitive pre-execution gate in adapter hooks.

## Prompt

`prompts/risk/llm_judge.md`. Inputs:

- operation summary,
- rendered command / plan,
- classification rationale,
- current risk config,
- user context (recent corrections, trust signals).

## Output Contract

```json
{
  "decision": "proceed" | "revise" | "abort",
  "confidence": 0.0..1.0,
  "concerns": ["..."],
  "mitigations": ["..."],
  "alternative_plan": "... | null"
}
```

## Semantics

- `proceed` — clear to execute.
- `revise` — execute after applying the listed mitigations.
- `abort` — upgrade to high-risk Council or escalate, depending on residual risk.

## Confidence Floor

`risk.config.judge_confidence_floor` (default 0.7). Below floor → upgrade.

## Model

- Default: cheapest capable model (Haiku / Gemini Flash).
- For mid-risk with high cost axis: upgrade to stronger model.

## Sandbox-Pair

For mid-risk operations supported by dry-run, Ciel runs the dry-run, feeds output to the judge, then proceeds on `proceed`. Dry-run + judge is preferred over judge alone.

## Logging

Every judge invocation logs to `activity.log` with:

- decision,
- confidence,
- model used,
- tokens,
- pre-existing classification summary.
