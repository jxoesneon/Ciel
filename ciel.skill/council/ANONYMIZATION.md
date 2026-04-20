# ANONYMIZATION

Stage 2 identity anonymization. Prevents inter-member bias. Borrowed from Karpathy / agent-council.

## Protocol

1. After Stage 1, Chairman has five votes: `{ coherence, capability, safety, efficiency, evolution }`.
2. Chairman assigns each an anonymous id from a stable set: `A, B, C, D, E`. Mapping is deterministic per run but scrambled across runs (seeded by `run_id`).
3. Chairman constructs a Stage 2 input bundle for each member containing the other four anonymized outputs only.
4. Prompts use `prompts/council/<lens>_stage2.md` which explicitly instructs each member not to attempt to de-anonymize.

## Why Anonymize

Members otherwise over-weight Safety's opinion by pattern, or reflexively disagree with Efficiency if they stereotype the role as "cut things". Anonymization forces each vote to stand on rationale.

## Is De-Anonymization Detectable

- Style fingerprints (e.g. Safety's tendency to mention "risk vectors") can leak identity.
- Chairman post-processes Stage 1 rationales to normalize stylistic tics where possible without changing meaning (`prompts/council/chairman_synthesis.md` has a pre-pass sub-prompt for this).
- Residual leakage is acceptable; perfect anonymization is not the goal — mitigation is.

## Audit

Anonymization mapping is logged in `~/.ciel/council/<run_id>/mapping.json` but not shared with members during the run. Accessible post-hoc for audit.

## Disabling

`configuration/global/council.config.md`:

```yaml
council:
  anonymize_stage2: true
```

Disabling is allowed for research / debugging only. Every run with anonymization disabled is flagged in `activity.log` for transparency.
