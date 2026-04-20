# COMPOSITION

Composing new skills from fragments instead of importing monolithic ones.

## When Ciel Composes

- The gap is a *combination* of existing skills with a small adapter layer.
- Tier 3 returned several partial matches but no single full match.
- Two mid-trust fragments can be combined into a higher-trust whole.

## Composition Units

A composition skill references its component skills:

```yaml
id: "deploy_static_site/SKILL.md"
version: 1.0.0
composes:
  - git/SKILL.md
  - shell/SKILL.md
  - api_client/SKILL.md
flow: |
  1. git/SKILL.md: checkout
  2. shell/SKILL.md: run build
  3. api_client/SKILL.md: upload to CDN
```

## Benefits

- Smaller L0 footprint (composition is description, not implementation duplication).
- Compositions inherit reliability from components.
- Changes to a component skill automatically improve compositions.

## Risks

- Composition failures can cascade. Trigger more aggressive outcome scoring.
- Coherence member checks composition doesn't cross scope boundaries (local-only components in a global composition, etc.).

## Execution

Router treats a composition as a plan. Each component step is subject to its own risk classification. A composition's overall classification is the max of its components.

## Self-Improvement

When the same ad-hoc sequence of 2+ skills recurs, Ciel proposes composing them into a new skill via `self_improvement/GLOBAL_IMPROVEMENT.md`.
