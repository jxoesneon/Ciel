---
name: security-bounty-hunter
version: 1.0.0
format: skill/1.0
description: An offensive security framework for discovering exploitable, bounty-worthy vulnerabilities in reachable surfaces.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:web"]
triggers:
  - pattern: "(hunt|discover).*(vulnerability|exploit|bounty)"
    confidence: 0.95
  - pattern: "find exploitable bugs"
    confidence: 0.9
---

# CIEL ADAPTATION: Security Bounty Hunter (Offensive Discovery)

This skill formalizes "Practical Vulnerability Hunting." Unlike a broad security review, it focuses on remotely reachable, user-controlled attack paths that qualify for responsible disclosure.

## In-Scope Patterns (The High-Value Sink)
- **SSRF**: User-controlled URLs reaching internal networks or cloud metadata.
- **Auth Bypass**: Logical flaws in middleware or API guards.
- **RCE**: Unsafe deserialization or upload-to-code-execution paths.
- **SQLi**: User input reaching a database sink without parameterization.
- **Path Traversal**: Arbitrary file read/write via unsanitized paths.

## Skip (Low-Signal)
- Local-only bugs (CLI-only `exec`).
- Missing security headers (informative only).
- Self-XSS.
- Demo or test-only code.

## Discovery Workflow
1. **Entrypoint ID**: Find all HTTP handlers, webhooks, and parsers.
2. **Triage**: Use `semgrep` or `grep_search` to find candidate sinks.
3. **Trace**: Read the code path end-to-end to verify user control.
4. **PoC**: Construct the smallest safe proof-of-concept (PoC) to verify exploitability.

## Anti-Patterns
- **Vibes-Hunting**: Claiming a bug exists without a working PoC.
- **Noisy Scans**: Reporting 50 "Low" severity lint warnings as "Bounties."
- **Scope Blindness**: Hunting on endpoints explicitly excluded in `SECURITY.md`.
