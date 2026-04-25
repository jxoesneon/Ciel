---
name: cicd_integration
version: 1.0.0
description: CI/CD pipeline interaction — trigger, status check, artifact retrieval.
triggers: [ci, cd, pipeline, github actions, gitlab ci, circle]
tags: [deploy, scope:both, runtime:any, risk:mid]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [api_client/SKILL.md, git/SKILL.md, secrets_manager/SKILL.md] }
---
# cicd_integration

Interact with CI systems.

## Operations

- `ci.trigger(workflow, ref, inputs?)`
- `ci.status(run_id)` — poll or subscribe.
- `ci.artifacts(run_id)` — list + download.
- `ci.logs(run_id, job?)`
- `ci.list_runs(branch?, status?)`.

## I/O Contract

```yaml
io_contract:
  input: { op, args }
  output: { result }
  idempotent: depends
  side_effects: [network]
```

## Providers

- GitHub Actions.
- GitLab CI.
- CircleCI, Buildkite, Drone (best-effort, detected from project).

Credentials resolved by `secrets_manager`.
