# BOOTSTRAP — Cold Start

First-ever run on a machine. `~/.ciel/` does not exist; MemPalace not installed.

## Seed State

Ciel ships with all seed skills bundled (`seed_skills/`). On cold start, Ciel has enough capability to install herself without needing the registry — she *is* the registry.

## Sequence

1. **Unpack** — if invoked from a `ciel.skill` archive, unpack to a tmp dir first.
2. **Mkdir** — create `~/.ciel/` with standard subdirs (see `memory/GLOBAL_STORE.md`).
3. **Install seed** — copy `seed_skills/` → `~/.ciel/skills/`.
4. **Write integrity** — compute SHA-256 of every tracked file → `~/.ciel/INTEGRITY.json`.
5. **Git init** — `GIT_SETUP.md`.
6. **Install MemPalace-rs** — `scripts/install.sh` runs `cargo install mempalace-rs --locked`. If rust missing, installs rust first (via `rustup-init.sh` with `-y` only if user consented at invocation; otherwise prompts).
7. **Create partitions** — `ciel-global` immediately; project partition at local init.
8. **Write default config** — instantiate `configuration/global/*.config.md` at defaults.
9. **Memory health** — `memory/HEALTH_CHECK.md`. On fail, fallback per `memory/FALLBACK.md`.
10. **Initial commit** — git commit `genesis: Ciel cold start @ <version>`.
11. **Announce** — identity summary + next-step hints to user.

## Single-Call Setup

`init/scripts/install.sh` (and `install.ps1` on Windows) bundles steps 2–10 into one invocation. The user runs once, gets a ready Ciel.

## Failure Handling

Any step failure:

- logs to `~/.ciel/bootstrap.log` (created even if git init failed),
- rolls back only the step that failed (not the whole bootstrap),
- escalates to user with a clear next-step.

## Seed-Skill Guarantee

Every seed skill listed in `seed_skills/SEED_SKILLS.md` is expected to be present and integrity-verified at the end of bootstrap. Missing = fatal; user must re-unpack.

## Idempotency

Re-running bootstrap on an already-initialized `~/.ciel/` is a no-op after step 2 detects the directory exists (defers to `INTEGRITY.md`).
