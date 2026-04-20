# SANDBOX

Isolated execution of unvalidated skill candidates.

## Isolation Layers

1. **Filesystem** — chroot / bind-mount to `~/.ciel/sandbox/<run_id>/`.
2. **Network** — default deny; allowlist only domains declared in the skill's `dependencies`.
3. **Shell** — restricted PATH; no privileged binaries (`sudo`, `systemctl`, `docker --privileged`, etc.).
4. **Resource limits** — CPU, memory, wall time (see `acquisition.config.md.sandbox_limits`).
5. **Secrets** — no real credentials; mock tokens injected per `seed_skills/secrets_manager/SKILL.md`.

## Backend

In order of preference:

1. **Docker / Podman** — if available.
2. **macOS Seatbelt** — sandbox-exec profile.
3. **Firejail / bubblewrap** — on Linux.
4. **chroot** — last resort.

Backend selection captured in trace. If no isolation backend is available, Ciel refuses to run Tier 3 acquisitions and escalates.

## Synthetic Inputs

`seed_skills/skill_builder/SKILL.md` generates representative inputs per the skill's `io_contract`. Fuzzing for side-effect-prone operations.

## Trace Capture

```yaml
sandbox_trace:
  run_id: ...
  backend: docker
  commands: [...]
  stdout_snippets: [...]
  stderr_snippets: [...]
  exit_codes: [...]
  network_attempts: [...]
  fs_writes: [...]
  duration_ms: ...
  anomalies: [...]
```

Attached to Council input. Safety member scrutinizes `network_attempts`, `fs_writes` outside sandbox, and exit codes.

## Cleanup

Sandbox directory retained for `sandbox_retention_hours` (default 48) then purged. Traces persist in MemPalace.
