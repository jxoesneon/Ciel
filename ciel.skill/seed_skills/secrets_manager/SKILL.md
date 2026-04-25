---
name: secrets_manager
version: 1.0.0
description: Secure credential handling — env vars, keychain, vault lookup; never plaintext.
triggers: [secret, credential, env var, keychain, vault]
tags: [security, scope:both, runtime:any, risk:high]
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [shell/SKILL.md] }
---

# secrets_manager

Resolve credentials by reference; never inline.

## Operations

- `sec.get(ref)` — returns a handle (opaque token). Handles are single-use by default.
- `sec.redact(text)` — scrub secret patterns.
- `sec.list_sources()` — env, keychain, vault types available.
- `sec.set(ref, source=keychain|env)` — store (never plaintext in files).

## I/O Contract

```yaml
io_contract:
  input: { op, ref, "args?" }
  output: { handle_or_ok }
  idempotent: depends
  side_effects: [keychain]
```

## Constraints (Constitutional)

- Raw secrets never touch activity.log, MemPalace, Council artifacts, or git.
- `sec.redact` runs at capture time on stdout/stderr flowing to storage.
- Refusal to exfiltrate: any operation that would transmit a secret to a network endpoint requires critical-risk escalation.
