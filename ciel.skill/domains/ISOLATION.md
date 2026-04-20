# ISOLATION — Cross-Project Guarantees

`locked: true` — Constitutional invariant.

## Guarantees

1. Project partitions never read from one another.
2. Project partitions never read from global except via explicit `lift(key)`.
3. Global partition never reads from a specific project except via `with_project(id)` scope — logged.
4. Learnings from project A never appear in project B unless promoted globally first AND generalized.
5. Deletion of a project's `.ciel/` and partition leaves no residue in other projects' data surfaces.

## Enforcement

- **Partition scoping** — `seed_skills/mempalace_manager/SKILL.md` rejects cross-partition reads without explicit scope declaration.
- **Activity log** — every cross-scope operation is logged with reason, actor, keys.
- **Integrity sweep** — periodic scan for stray cross-references; violations are immediate Safety incidents.
- **Promotion stripping** — `HARMONIZATION.md` style identifier removal at promotion time.

## Common Leakage Vectors (defended against)

- Absolute paths in traces — stripped at promotion; hashed at storage time.
- Repo names in prompts — same.
- Environment secrets captured in stdout — `secrets_manager` redacts at capture time; never in MemPalace raw.
- Git remote URLs — stripped or hashed when crossing scopes.
- API endpoints (internal URLs) — anonymized per scope policy.

## Testing

Integrity sweep includes a probe that synthesizes a lookup with project-A identifiers against project-B partition; must return empty. Any hit is an immediate escalation.

## User Controls

- `/ciel-purge <project>` — nuke a project's data.
- `/ciel-sweep --cross-scope` — explicit cross-scope audit.
- `configuration/global/memory.config.md.isolation_strict` — if true (default), any cross-scope read without explicit scope declaration fails loudly rather than falling back to a guess.
