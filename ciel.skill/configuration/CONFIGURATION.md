# CONFIGURATION — Philosophy

Vast surface. Maximally granular. Each concern in its own file. Ciel tunes her own config at user request, gated by the Council.

## Layout

- `configuration/global/` — cross-project defaults (9 files).
- `configuration/local/` — project-specific overrides (4 files).

Each file targets a single concern. A user (or Ciel) finds the right knob by walking the directory rather than reading a monolithic config.

## Hierarchy

Effective config = defaults → global overrides → local overrides → CLI flags. Later wins. Constitutional floors cap the effective value.

## Tuning

Changes routed through `TUNING.md` → Council for non-trivial; auto-apply for trivial (in-range).

## Source of Truth

`configuration/global/*.config.md` files carry canonical YAML blocks. Ciel reads the YAML block and produces an effective config object. Prose around the block explains rationale and boundaries.

## Schema

Every field is described in `SCHEMA.md` with type, default, legal range, and mutability (Constitutional / tunable / local-only).

## Defaults

`DEFAULTS.md` holds the canonical defaults as a single YAML document for bootstrap and fallback.
