---
name: dependency_audit
version: 1.0.0
description: Vulnerability scanning, outdated-dep detection, update strategy.
triggers: [audit deps, cve, vulnerability, outdated]
tags: [security, pkg, scope:both, runtime:any, risk:low]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [package_manager/SKILL.md, shell/SKILL.md] }
---

# dependency_audit

Scan project and acquired skills for vulnerable or outdated dependencies.

## Operations

- `audit.scan(path)` — aggregated across ecosystems.
- `audit.by_severity(path, min)` — filter.
- `audit.updates(path)` — available update paths.
- `audit.explain(cve_id)` — fetch CVE details with `web_fetch`.

## I/O Contract

```yaml
io_contract:
  input: { op, path, "min_severity?" }
  output: { findings: [ { pkg, version, cve, severity, fix_version } ] }
  idempotent: true
  side_effects: [network]
```

## Backends

Calls `npm audit`, `pip-audit`, `cargo audit`, `bundle audit`, `govulncheck` as available. Cross-checks with OSV database via `web_fetch`.

## Policy

`acquisition.config.cve_threshold` blocks skill integration when exceeded.
