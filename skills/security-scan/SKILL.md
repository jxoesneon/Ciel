---
name: security-scan
version: 1.0.0
format: skill/1.0
description: Audits CIEL configuration for misconfigurations and prompt injection risks using AgentShield.
runtimes: ["claude_code", "gemini_cli", "windsurf", "generic"]
license: MIT
tags: ["ciel", "harmonized", "domain:ai"]
triggers:
  - pattern: "(scan|audit).*(config|settings|shield)"
    confidence: 0.95
  - pattern: "npx ecc-agentshield"
    confidence: 1.0
---

# CIEL ADAPTATION: Security Scan (Harness Auditing)

This skill formalizes the auditing of the CIEL harness itself. It uses `AgentShield` to detect overly permissive permissions, hardcoded secrets in configs, and command injection risks in hooks.

## Scan Targets
- **CLAUDE.md**: Search for auto-run instructions that could be exploited via prompt injection.
- **Settings**: Identify wildcard shell permissions (`Bash(*)`) or missing deny lists.
- **MCP Config**: Detect hardcoded env secrets or risky third-party servers.
- **Hooks**: Audit shell scripts for `${file}` interpolation without proper quoting.

## Operational Modes
1. **Sanity Check**: Run `npx ecc-agentshield scan` before committing config changes.
2. **Deep Analysis (Adversarial)**: Use the 3-agent pipeline (Attacker, Defender, Auditor) for complex security decisions.
3. **Scaffold**: Use `agentshield init` to generate a secure-by-default environment.

## Grading Scale
- **A (90+)**: Secure configuration.
- **C-D (40-74)**: Needs immediate attention.
- **F (<40)**: CRITICAL vulnerabilities (e.g., hardcoded tokens).

## Anti-Patterns
- **Silent Errors**: Using `2>/dev/null` in security hooks.
- **Unpinned Tools**: Installing MCP servers via `npx` without pinning the version.
- **Context Bypass**: Adding instructions to `CLAUDE.md` that tell agents to ignore security rules.
