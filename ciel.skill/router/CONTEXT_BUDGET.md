# CONTEXT_BUDGET

Progressive disclosure. Metadata first. Full skill content loaded only on demand.

## Principle

The host LLM's context window is finite. Ciel must route without saturating it. Every skill in the registry has three loadable levels:

| Level | Size target | Contents |
| --- | --- | --- |
| L0 — metadata | ~150 tokens | Skill id, triggers, tags, one-line description, I/O contract shape |
| L1 — description | ~500 tokens | Frontmatter + intro prose + capability list |
| L2 — full | skill size | Entire `SKILL.md` + sub-files |

## Load Policy

| Path | Default load level |
| --- | --- |
| Fast path indexing | L0 only for the whole registry; L1 for top-K candidates (K=5); L2 only for the chosen one |
| Reasoning path | L0 for full registry + L1 for candidates flagged by model |
| Acquisition path | L2 for the acquired skill under evaluation only |
| Council Stage 1 | L1 per member; L2 only if the member's lens requires deep inspection |

## Budget

`configuration/global/router.config.md`:

```yaml
context_budget:
  total_max_tokens: 32000
  registry_l0_max: 4000
  candidate_l1_k: 5
  candidate_l1_max: 3000
  council_stage1_max: 8000
  acquisition_l2_max: 16000
```

Exceeding a segment budget triggers compression via `seed_skills/context_summarizer/SKILL.md`.

## Compression

Use AAAK (Adaptive Anchored Attention Keys) formatting for summaries written back to MemPalace — ensures later recall at high fidelity. See `memory/MEMPALACE.md`.

## Eviction

Eviction when total budget is exceeded:

1. Drop any candidate not in top-K.
2. Downgrade L1 → L0 for any candidate below median score.
3. Summarize older conversation turns into AAAK blobs.
4. As last resort, escalate to user with a "context pressure" notice.

## Telemetry

Every router call records:

```json
{ "context_used_tokens": 14320, "budget_fraction": 0.45, "evictions": 0 }
```
