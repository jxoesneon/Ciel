# CLASSIFICATION

`locked: true`.

## Axes

Every operation is scored 0–10 on each axis. Classification uses weighted max.

| Axis | Weight | Meaning |
| --- | --- | --- |
| reversibility | 0.25 | Can we undo this? (git revert, restore from backup) |
| blast_radius | 0.20 | What's the worst case if it fails? |
| external_impact | 0.20 | Touches systems outside local dev env? |
| data_sensitivity | 0.15 | Involves secrets, PII, PHI, PCI? |
| cost | 0.10 | Monetary or compute cost? |
| novelty | 0.10 | Has Ciel done this successfully before? |

`axis_risk ∈ 0..10`.

`composite = Σ weight_i * axis_i`.

## Levels

- `composite < mid_threshold` (default 3.0) → **low**
- `mid_threshold ≤ composite < high_threshold` (default 3.0–6.0) → **mid**
- `high_threshold ≤ composite < critical_threshold` (default 6.0–8.5) → **high**
- `composite ≥ critical_threshold` (default 8.5) OR any veto-condition match → **critical**

Veto conditions include: irreversible + external_impact ≥ 8, data_sensitivity ≥ 9 without consent, license boundary violation, any item in `council/rubrics/VETO_CONDITIONS.md`.

## Examples

| Operation | Level |
| --- | --- |
| `ls` in project root | low |
| Read a file | low |
| Edit a project source file | low |
| `git commit` | low |
| `git push origin main` | mid |
| Install a system package | mid |
| Run a new untrusted skill in sandbox | mid |
| Install MCP server | high |
| Modify locked core file | high |
| `git push --force` shared branch | critical |
| `npm publish` to public registry | critical |
| Send an email / external API with state mutation | critical |
| `rm -rf` outside sandbox | critical |
| Update production database | critical |

## Classification Prompt

Automated via `prompts/risk/classification.md` when the router cannot match the operation to an obvious category.
