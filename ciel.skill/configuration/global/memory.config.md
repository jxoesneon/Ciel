# memory.config — Global Memory

```yaml

# <anchor:start>

memory:
  backend: custom              # obsidian (custom) | mempalace | sqlite | filesystem
  auto_update: false
  version_pin: null
  reinstall_check_days: 7
  health_check_interval_minutes: 60
  isolation_strict: true       # Constitutional: locked true
  partition_size_limit_mb: 1024
  fallback_snapshot_retention_days: 30
  custom:
    entry: "ciel.skill/memory/backends/obsidian/cli.mjs"
    runtime: node
    endpoint: null
    auth_env: OBSIDIAN_API_KEY

# <anchor:end>

```

## Notes

- `backend` — default `custom` (Obsidian vault) on the `Obsidian` branch. Change requires Council (structural).
- `isolation_strict: true` is Constitutional. Cannot be disabled.
- `auto_update` — unused for the Obsidian backend; vault contents are live and versioned by git.
- `version_pin` — unused for the Obsidian backend.

## Alternative: SQLite/Filesystem Backend

To use the Obsidian vault as Ciel's brain, switch to `backend: custom` and point the entry to the Obsidian adapter:

```yaml
memory:
  backend: custom
  auto_update: false
  isolation_strict: true
  custom:
    entry: "ciel.skill/memory/backends/obsidian/cli.mjs"
    runtime: node
    endpoint: null
    auth_env: OBSIDIAN_API_KEY
```

Required environment variables:

- `OBSIDIAN_API_URL` — default `http://127.0.0.1:27123`
- `OBSIDIAN_API_KEY` — Bearer token from Obsidian Local REST API settings
- `OBSIDIAN_VAULT_PATH` — absolute path to the `obsidian-brain` vault
- `OBSIDIAN_HYBRID_SEARCH_URL` — default `http://127.0.0.1:3939`

Run the adapter self-test before switching:

```bash
node ciel.skill/memory/backends/obsidian/cli.mjs --self-test
```

See `ciel.skill/memory/backends/obsidian/README.md` for the full migration path.

## Fallback Order

Hard-coded: Obsidian (custom) → SQLite → Filesystem. See `memory/FALLBACK.md`.
