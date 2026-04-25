# MANIFEST

## Format

- **Spec**: `skill/1.0`
- **Container**: ZIP (`.skill` is a renamed `.zip`)
- **Encoding**: UTF-8
- **Line endings**: LF

## Identity

- **Name**: `ciel`
- **Version**: `1.0.0`
- **Domain**: orchestration / meta-skill
- **Chairman**: self (`council/CHAIRMAN.md`)

## Declared Runtime Support

| Runtime | Adapter | Status |
| --- | --- | --- |
| Claude Code | `adapters/claude_code/` | full |
| Gemini CLI | `adapters/gemini_cli/` | full |
| Windsurf | `adapters/windsurf/` | full |
| Generic | `adapters/generic/` | probe-and-adapt |

## Runtime Dependencies

| Tool | Source | Required | Auto-install |
| --- | --- | --- | --- |
| `mempalace-rs` | `cargo install mempalace-rs` | primary memory backend | yes (see `memory/INSTALL.md`) |
| `git` | system | versioning of `~/.ciel/` | yes (verified, not installed) |
| `curl` / `wget` | system | web acquisition tier | yes (verified) |
| `sqlite3` | system | fallback memory backend | optional |
| `rustc` / `cargo` | `rustup` | builds `mempalace-rs` | yes (installed if missing) |
| `node` / `npm` | system | skill-runner pkg ops | optional |
| `python3` / `pip` | system | skill-runner pkg ops | optional |
| `docker` | system | acquisition sandboxing | optional |

## Declared Capability Surface

- Skill routing (hybrid)
- Skill registry (persistent, git-versioned)
- Skill acquisition (tiered: registry / MCP / web)
- Skill composition + harmonization
- Council of Five deliberation
- Two-domain operation (global + local)
- MemPalace-rs primary memory with SQLite / filesystem fallback
- Self-improvement loop with git-backed rollback
- Risk classification + LLM judge gate
- OpenTelemetry + activity log observability

## Checksum Strategy

On build, `init/scripts/install.sh` writes `~/.ciel/INTEGRITY.json` with SHA-256 of every tracked file. `init/INTEGRITY.md` defines verification protocol on re-init.

## Subfile Index (top-level only)

```text
SKILL.md, MANIFEST.md, CHANGELOG.md
core/          router/         adapters/      council/
registry/      acquisition/    memory/        init/
self_improvement/  risk/       domains/       observability/
configuration/  prompts/       seed_skills/   templates/
```

Total tracked files: **~246**.
 See each directory's index `*.md` for contents.

## Locked Core

Files under `core/CONSTITUTION.md` and any file that Constitution references as `locked: true` cannot be mutated without:

1. Council of Five majority approval (Safety non-veto), AND
2. Explicit user confirmation.

## Licensing

- Ciel source: MIT
- Acquired skills: each carries its own `LICENSE.md`. Ciel refuses to integrate any skill without a compatible license (see `council/members/SAFETY.md`).
