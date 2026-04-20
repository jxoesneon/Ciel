---
name: test_runner
version: 1.0.0
description: Test execution, coverage analysis, failure diagnosis, re-run strategy.
triggers: [test, run tests, pytest, jest, cargo test, go test]
tags: [test, scope:both, runtime:any, risk:low]
runtime_compatibility: { claude_code: true, gemini_cli: true, generic: true }
license: Apache-2.0
source: { tier: 0, origin: seed }
dependencies: { skills: [shell/SKILL.md, environment_detection/SKILL.md] }
---

# test_runner

Execute and analyze tests.

## Operations

- `test.run(target?, filter?)` — run tests; returns pass/fail/error counts + failure details.
- `test.coverage(target?)` — coverage report with file-level breakdown.
- `test.diagnose(failure)` — suggest root cause from stack trace + recent diffs.
- `test.rerun(strategy)` — retry-only-failing, full, or bisect-over-commits.

## I/O Contract

```yaml
io_contract:
  input: { op, "target?", "filter?", "coverage?" }
  output: { summary: { passed, failed, skipped }, failures: [ { name, trace } ], "coverage?" }
  idempotent: partial
  side_effects: [shell, fs]
```

## Framework Detection

Auto-detects pytest, jest, mocha, cargo test, go test, rspec, phpunit, dotnet test, JUnit, etc. Respects project test config.
