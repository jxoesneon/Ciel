---
name: ciel-root-cause-debugger
version: 1.0.0
format: skill/1.0
description: CIEL's framework for systematic error tracing, log analysis, and empirical bug fixing.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:systems"]
triggers:
  - pattern: "(debug|fix|trace|error|crash|fail).*(root cause|log|stack)"
    confidence: 0.9
  - pattern: "analyze logs"
    confidence: 1.0

source: { tier: 1, origin: harmonized }
dependencies: { skills: [], mcp: [], system: [] }
---
# CIEL ADAPTATION: Root Cause Debugger (The Scalpel)

This skill formalizes the transition from "guessing" to "empirical evidence" during debugging. it mandates the use of execution data.

## Debugging Protocol (RCA)
1. **Reproduce**: Create a minimal reproduction script or test case. If it doesn't fail, you haven't found the bug.
2. **Trace**: Use `run_shell_command` to execute the code and capture `stdout/stderr`.
3. **Isolate**: Use `grep_search` to find the failure point in the call stack.
4. **Fix**: Apply the minimal change needed to pass the reproduction test.
5. **Verify**: Run the entire suite to ensure no regressions.

## Tooling Usage
- **Logs**: Read logs with `read_file` or `read_background_output`. Do not assume log content.
- **System**: Use `systemctl`, `journalctl`, or `docker logs` to check environment health.
- **Profiling**: Use `perf`, `time`, or language-specific profilers to identify bottlenecks.

## Anti-Patterns
- **The Guess-and-Check**: Modifying code before reproducing the failure.
- **The Silent Swallow**: Adding a try-catch block that hides the error instead of fixing the cause.
- **Amnesiac Debugging**: Fixing a bug without adding a regression test.
