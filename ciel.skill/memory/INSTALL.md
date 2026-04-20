# INSTALL — MemPalace-rs

Ciel installs, maintains, and updates MemPalace-rs.

## Prerequisites

- Rust toolchain (`rustup`, `cargo`). If missing, Ciel proposes installation via `seed_skills/package_manager/SKILL.md`.

## Install Command

```bash
cargo install mempalace-rs --locked
```

Success verified by `mempalace-rs --version` returning a parseable semver.

## Upgrade Cadence

On every `init/INIT.md` invocation:

1. Query installed version.
2. Query latest (`cargo search mempalace-rs` or `crates.io` API).
3. If newer is available and `memory.config.auto_update: true`, propose upgrade.
4. Upgrade is Council-gated (`council/invocation_scopes/SKILL_INTEGRATION.md`) because it changes a load-bearing dep.
5. On pass, run upgrade, migration, and integrity check.

## Migration

Schema version lives at `ciel-global:meta/schema_version`. Each upgrade runs migrations in order. Failed migration → auto-rollback to previous version and backup restore (`BACKUP.md`).

## Fallback Installation Failure

If installation fails (no rust, offline, crates.io unreachable) or MemPalace health check fails, Ciel falls back per `FALLBACK.md`:

1. Try SQLite backend (`backends/SQLITE.md`).
2. Try filesystem KV (`backends/FILESYSTEM.md`).
3. Inform user; continue in degraded mode.

## User Overrides

```yaml
memory:
  backend: mempalace|sqlite|filesystem|custom
  auto_update: true
  version_pin: null          # or "1.2.3"
```

Pinning prevents auto-updates; Ciel warns on known-security-update version gaps.

## Binary Location

- macOS/Linux: `$CARGO_HOME/bin/mempalace-rs` (typically `~/.cargo/bin/mempalace-rs`).
- Windows: `%USERPROFILE%\.cargo\bin\mempalace-rs.exe`.

`init/scripts/install.sh` ensures `$CARGO_HOME/bin` is on PATH.
